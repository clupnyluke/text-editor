use crossterm::cursor::{
    position, MoveDown, MoveLeft, MoveRight, MoveTo, MoveToColumn, MoveUp, RestorePosition,
    SavePosition, SetCursorStyle,
};
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, Clear, ClearType, DisableLineWrap, EnableLineWrap,
};
use crossterm::{execute, queue};
use std::io::stdout;

use super::buffer::Buffer;
use super::{screen, IOResult};

enum EditorMode {
    Insert,
    Control,
    Command,
}
pub struct Controller {
    should_quit: bool,
    command_text: String,
    mode: EditorMode,
}

impl Default for Controller {
    fn default() -> Self {
        Controller {
            should_quit: false,
            command_text: String::new(),
            mode: EditorMode::Control,
        }
    }
}

impl Controller {
    pub fn new() -> Self {
        Controller::default()
    }

    fn set_mode(&mut self, mode: EditorMode) -> IOResult {
        self.mode = mode;
        match self.mode {
            EditorMode::Insert => {
                execute!(stdout(), SetCursorStyle::BlinkingBar)?;
            }
            EditorMode::Control => {
                execute!(stdout(), SetCursorStyle::BlinkingBlock)?;
                self.command_text.clear();
            }
            EditorMode::Command => {
                queue!(stdout(), SavePosition, SetCursorStyle::BlinkingBar)?;
                screen::update_command_text(&self.command_text)?;
            }
        }
        Ok(())
    }

    pub fn init(&mut self, buffer: &Buffer) -> IOResult {
        queue!(
            stdout(),
            Clear(ClearType::All),
            DisableLineWrap,
            SetCursorStyle::BlinkingBlock
        )?;
        screen::print_all(buffer)?;
        execute!(stdout(), MoveTo(0, 0))?;

        enable_raw_mode()?;
        Ok(())
    }

    pub fn handle_input(&mut self, buffer: &mut Buffer) -> IOResult {
        loop {
            if self.should_quit {
                break;
            }
            if let Event::Key(event) = read()? {
                if KeyCode::Char('q') == event.code && event.modifiers == KeyModifiers::CONTROL {
                    self.should_quit = true;
                }
                if KeyCode::Esc == event.code {
                    self.set_mode(EditorMode::Control)?;
                }

                match self.mode {
                    EditorMode::Insert => self.handle_input_insert_mode(buffer, event)?,
                    EditorMode::Control => self.handle_input_control_mode(buffer, event)?,
                    EditorMode::Command => self.handle_input_command_mode(buffer, event)?,
                }
            }
        }
        Ok(())
    }

    fn handle_input_insert_mode(&mut self, buffer: &mut Buffer, event: KeyEvent) -> IOResult {
        let (x, y) = position()?;
        match event.code {
            KeyCode::Char(char) => buffer.insert_char_on_line(char, y, x)?,
            KeyCode::Delete => {
                let line = buffer.get_line(y).unwrap();
                if x as usize >= line.len() && (y as usize) == buffer.len() {
                    return Ok(());
                }
                if x as usize >= line.len() {
                    // DELETE AT END OF LINE AND NOT AT EOF
                    buffer.move_line_contents_up_one_row(y + 1);
                    buffer.delete_line(y + 1);
                    screen::update_line_until_eof(buffer, y)?;
                } else {
                    buffer.delete_char_on_line(y, x)?;
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
                        MoveTo(buffer.get_line(y - 1).unwrap().len() as u16, y - 1)
                    )?;
                    buffer.move_line_contents_up_one_row(y);
                    buffer.delete_line(y);
                    screen::update_line_until_eof(&buffer, y - 1)?;
                } else {
                    queue!(stdout(), MoveLeft(1))?;
                    buffer.delete_char_on_line(y, x - 1)?;
                }
            }
            KeyCode::Enter => {
                let line = buffer.get_line_mut(y).unwrap();
                let append_line = line.split_off(x as usize);
                buffer.insert_line(y + 1, append_line);
                queue!(stdout(), MoveDown(1), MoveToColumn(0))?;
                screen::update_line_until_eof(&buffer, y)?;
            }
            _ => (),
        }
        Ok(())
    }

    fn handle_input_control_mode(&mut self, buffer: &mut Buffer, event: KeyEvent) -> IOResult {
        let (x, y) = position()?;
        match event.code {
            KeyCode::Char(char) => match char {
                'h' => {
                    if x > 0 {
                        execute!(stdout(), MoveLeft(1))?;
                    }
                }
                'j' => {
                    if (y as usize) < buffer.len() - 1 {
                        execute!(stdout(), MoveDown(1))?;
                        self.snap_to_line_end(buffer)?;
                    }
                }
                'k' => {
                    if y > 0 {
                        execute!(stdout(), MoveUp(1))?;
                        self.snap_to_line_end(buffer)?;
                    }
                }
                'l' => {
                    if (x as usize)
                        < usize::max(buffer.get_line(y).unwrap_or(&String::new()).len(), 1) - 1
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
                ':' | '\\' => {
                    self.command_text.insert(0, char);
                    self.set_mode(EditorMode::Command)?;
                }

                _ => (),
            },
            _ => (),
        }
        Ok(())
    }

    fn handle_input_command_mode(&mut self, buffer: &mut Buffer, event: KeyEvent) -> IOResult {
        match event.code {
            KeyCode::Char(char) => {
                self.command_text.insert(self.command_text.len(), char);
                screen::update_command_text(&self.command_text)?;
            }
            KeyCode::Delete => {
                let (x, _y) = position()?;
                if (x as usize) < self.command_text.len() {
                    self.command_text.remove(x as usize);
                    screen::update_command_text(&self.command_text)?;
                }
                if self.command_text.len() == 0 {
                    self.exit_command_mode(buffer)?;
                }
            }
            KeyCode::Backspace => {
                let (x, _y) = position()?;
                if x as i32 - 1 < 0 {
                    return Ok(());
                }
                queue!(stdout(), MoveLeft(1))?;
                self.command_text.remove((x as usize) - 1);
                screen::update_command_text(&self.command_text)?;
                if self.command_text.len() == 0 {
                    self.exit_command_mode(buffer)?;
                }
            }
            KeyCode::Enter => {
                //EXECUTE
            }
            _ => (),
        }
        Ok(())
    }

    fn snap_to_line_end(&self, buffer: &mut Buffer) -> IOResult {
        let (x, y) = position()?;
        let line_end = u16::max(buffer.get_line(y).unwrap().len() as u16, 1);
        if x > line_end - 1 {
            execute!(stdout(), MoveTo(line_end - 1, y))?
        }
        Ok(())
    }

    fn exit_command_mode(&mut self, buffer: &Buffer) -> IOResult {
        self.set_mode(EditorMode::Control)?;
        execute!(stdout(), RestorePosition, SetCursorStyle::BlinkingBlock)?;
        screen::update_line(buffer, size()?.1)?;
        Ok(())
    }

    pub fn terminate(&mut self) -> IOResult {
        execute!(stdout(), EnableLineWrap)?;
        disable_raw_mode()?;
        Ok(())
    }
}
