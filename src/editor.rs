use buffer::Buffer;
use crossterm::cursor::{
    position, MoveDown, MoveLeft, MoveRight, MoveTo, MoveToColumn, MoveUp, SetCursorStyle,
};
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, DisableLineWrap, EnableLineWrap,
};
use crossterm::{execute, queue};
use std::io::stdout;

mod buffer;
mod screen;

enum EditorMode {
    Insert,
    Control,
    Command,
}

pub struct Editor<'a> {
    should_quit: bool,
    mode: EditorMode,
    buffer: Buffer<'a>,
}

impl<'a> Default for Editor<'a> {
    fn default() -> Self {
        Editor {
            should_quit: false,
            mode: EditorMode::Control,
            buffer: Buffer::new(),
        }
    }
}

impl<'a> Editor<'a> {
    pub fn new(file_path: Option<&'a String>) -> Self {
        let mut editor = Editor::default();
        if let Err(err) = editor.buffer.read_file(file_path) {
            panic!("{err:#?}")
        }
        editor
    }

    fn set_mode(&mut self, mode: EditorMode) -> Result<(), std::io::Error> {
        self.mode = mode;
        match self.mode {
            EditorMode::Insert => {
                execute!(stdout(), SetCursorStyle::BlinkingBar)?;
            }
            EditorMode::Control => {
                execute!(stdout(), SetCursorStyle::BlinkingBlock)?;
            }
            EditorMode::Command => (),
        }
        Ok(())
    }

    pub fn run(&mut self) {
        if let Err(err) = self.repl() {
            panic!("{err:#?}")
        }
    }

    fn init(&mut self) -> Result<(), std::io::Error> {
        queue!(
            stdout(),
            Clear(ClearType::All),
            DisableLineWrap,
            SetCursorStyle::BlinkingBlock
        )?;
        screen::print_all(&self.buffer)?;
        execute!(stdout(), MoveTo(0, 0))?;

        enable_raw_mode()?;
        Ok(())
    }

    fn repl(&mut self) -> Result<(), std::io::Error> {
        self.init()?;
        loop {
            if self.should_quit {
                break;
            }
            self.handle_input_master()?;
        }
        self.terminate()?;
        Ok(())
    }

    fn terminate(&mut self) -> Result<(), std::io::Error> {
        execute!(stdout(), EnableLineWrap)?;
        disable_raw_mode()?;
        Ok(())
    }

    fn handle_input_master(&mut self) -> Result<(), std::io::Error> {
        if let Event::Key(event) = read()? {
            if KeyCode::Char('q') == event.code && event.modifiers == KeyModifiers::CONTROL {
                self.should_quit = true;
            }
            if KeyCode::Esc == event.code {
                self.set_mode(EditorMode::Control)?;
            }

            match self.mode {
                EditorMode::Insert => self.handle_input_insert_mode(event)?,
                EditorMode::Control => self.handle_input_control_mode(event)?,
                EditorMode::Command => self.handle_input_command_mode(event)?,
            }
        }
        Ok(())
    }

    fn handle_input_insert_mode(&mut self, event: KeyEvent) -> Result<(), std::io::Error> {
        let (x, y) = position()?;
        match event.code {
            KeyCode::Char(char) => self.buffer.insert_char_on_line(char, y, x)?,
            KeyCode::Delete => {
                let line = self.buffer.get_line(y).unwrap();
                if x as usize >= line.len() && (y as usize) == self.buffer.len() {
                    return Ok(());
                }
                if x as usize >= line.len() {
                    // DELETE AT END OF LINE AND NOT AT EOF
                    self.buffer.move_line_contents_up_one_row(y + 1);
                    self.buffer.delete_line(y + 1);
                    screen::update_line_until_eof(&self.buffer, y)?;
                } else {
                    self.buffer.delete_char_on_line(y, x)?;
                }
                ()
            }
            KeyCode::Backspace => {
                if x as i32 - 1 < 0 && y == 0 {
                    return Ok(());
                }
                if x as i32 - 1 < 0 {
                    // BACKSPACE AT BEGINNING OF LINE AND NOT AT START OF FILE
                    queue!(
                        stdout(),
                        MoveTo(self.buffer.get_line(y - 1).unwrap().len() as u16, y - 1)
                    )?;
                    self.buffer.move_line_contents_up_one_row(y);
                    self.buffer.delete_line(y);
                    screen::update_line_until_eof(&self.buffer, y - 1)?;
                } else {
                    queue!(stdout(), MoveLeft(1))?;
                    self.buffer.delete_char_on_line(y, x - 1)?;
                }
            }
            KeyCode::Enter => {
                let line = self.buffer.get_line_mut(y).unwrap();
                let append_line = line.split_off(x as usize);
                self.buffer.insert_line(y + 1, append_line);
                queue!(stdout(), MoveDown(1), MoveToColumn(0))?;
                screen::update_line_until_eof(&self.buffer, y)?;
            }
            _ => (),
        }
        Ok(())
    }

    fn handle_input_control_mode(&mut self, event: KeyEvent) -> Result<(), std::io::Error> {
        let (x, y) = position()?;
        match event.code {
            KeyCode::Char(char) => match char {
                'h' => {
                    if x > 0 {
                        execute!(stdout(), MoveLeft(1))?;
                    }
                }
                'j' => {
                    if (y as usize) < self.buffer.len() - 1 {
                        execute!(stdout(), MoveDown(1))?;
                        self.snap_to_line_end()?;
                    }
                }
                'k' => {
                    if y > 0 {
                        execute!(stdout(), MoveUp(1))?;
                        self.snap_to_line_end()?;
                    }
                }
                'l' => {
                    if (x as usize)
                        < usize::max(self.buffer.get_line(y).unwrap_or(&String::new()).len(), 1) - 1
                    {
                        execute!(stdout(), MoveRight(1))?;
                    }
                }
                'i' => {
                    self.set_mode(EditorMode::Insert)?;
                }
                'a' => {
                    execute!(stdout(), MoveRight(1))?;
                    self.set_mode(EditorMode::Insert)?;
                }
                _ => (),
            },
            _ => (),
        }
        Ok(())
    }

    fn snap_to_line_end(&self) -> Result<(), std::io::Error> {
        let (x, y) = position()?;
        let line_end = u16::max(self.buffer.get_line(y).unwrap().len() as u16, 1);
        if x > line_end - 1 {
            execute!(stdout(), MoveTo(line_end - 1, y))?
        }
        Ok(())
    }

    fn handle_input_command_mode(&mut self, event: KeyEvent) -> Result<(), std::io::Error> {
        Ok(())
    }
}
