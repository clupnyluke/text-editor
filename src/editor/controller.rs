use crossterm::cursor::{MoveLeft, RestorePosition, SavePosition, SetCursorStyle};
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::{execute, queue};
use std::io::stdout;

use super::buffer::Buffer;
use super::terminal::Terminal;
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

    pub fn init(&mut self, buffer: &Buffer, terminal: &mut Terminal) -> IOResult {
        terminal.goto(buffer, 0, 0)?;
        screen::update_line_until_eof(buffer, terminal, 0)?;
        Ok(())
    }

    pub fn handle_input(&mut self, buffer: &mut Buffer, terminal: &mut Terminal) -> IOResult {
        loop {
            if self.should_quit {
                break;
            }
            if let Event::Key(event) = read()? {
                if KeyCode::Char('q') == event.code && event.modifiers == KeyModifiers::CONTROL {
                    self.should_quit = true;
                }
                if KeyCode::Char('s') == event.code && event.modifiers == KeyModifiers::CONTROL {
                    buffer.write_file(None)?;
                }
                if KeyCode::Esc == event.code {
                    self.set_mode(EditorMode::Control)?;
                }

                match self.mode {
                    EditorMode::Insert => self.handle_input_insert_mode(buffer, terminal, event)?,
                    EditorMode::Control => {
                        self.handle_input_control_mode(buffer, terminal, event)?
                    }
                    EditorMode::Command => {
                        self.handle_input_command_mode(buffer, terminal, event)?
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_input_insert_mode(
        &mut self,
        buffer: &mut Buffer,
        terminal: &mut Terminal,
        event: KeyEvent,
    ) -> IOResult {
        let (x, y) = terminal.virtual_cursor();
        match event.code {
            KeyCode::Char(char) => buffer.insert_char_on_line(terminal, char, y, x)?,
            KeyCode::Delete => {
                let line = buffer.get_line(y).unwrap();
                if x >= line.len() && y == buffer.len() {
                    return Ok(());
                }
                if x >= line.len() {
                    buffer.move_line_contents_up_one_row(y + 1)?;
                    buffer.delete_line(y + 1)?;
                    screen::update_line_until_eof(buffer, terminal, y)?;
                } else {
                    buffer.delete_char_on_line(terminal, y, x)?;
                }
                ()
            }
            KeyCode::Backspace => {
                if x < 1 {
                    if y == 0 {
                        return Ok(());
                    }
                    terminal.goto(buffer, buffer.get_line(y - 1).unwrap().len(), y)?;
                    buffer.move_line_contents_up_one_row(y)?;
                    buffer.delete_line(y)?;
                    terminal.move_up(buffer)?;
                    screen::update_line_until_eof(buffer, terminal, y - 1)?;
                } else {
                    terminal.move_left(buffer)?;
                    buffer.delete_char_on_line(terminal, y, x - 1)?;
                }
            }
            KeyCode::Enter => {
                let line = buffer.get_line_mut(y).unwrap();
                let append_line = line.split_off(x);
                buffer.insert_line(y + 1, append_line)?;
                terminal.move_down(buffer)?;
                terminal.goto_beginning_of_line(buffer)?;
                screen::update_line_until_eof(buffer, terminal, y)?;
            }
            KeyCode::Down => terminal.move_down(buffer)?,
            KeyCode::Up => terminal.move_up(buffer)?,
            KeyCode::Left => terminal.move_left(buffer)?,
            KeyCode::Right => terminal.move_right_for_insert(buffer)?,
            _ => (),
        }
        Ok(())
    }

    fn handle_input_control_mode(
        &mut self,
        buffer: &mut Buffer,
        terminal: &mut Terminal,
        event: KeyEvent,
    ) -> IOResult {
        match event.code {
            KeyCode::Char(char) => match char {
                'h' => {
                    terminal.move_left(buffer)?;
                }
                'j' => {
                    terminal.move_down(buffer)?;
                }
                'k' => {
                    terminal.move_up(buffer)?;
                }
                'l' => {
                    terminal.move_right(buffer)?;
                }
                'i' => {
                    self.set_mode(EditorMode::Insert)?;
                }
                'a' => {
                    terminal.move_right_for_insert(buffer)?;
                    self.set_mode(EditorMode::Insert)?;
                }
                ':' | '\\' => {
                    self.command_text.clear();
                    self.command_text.insert(0, char);
                    self.set_mode(EditorMode::Command)?;
                }

                _ => (),
            },
            KeyCode::Down => terminal.move_down(buffer)?,
            KeyCode::Up => terminal.move_up(buffer)?,
            KeyCode::Left => terminal.move_left(buffer)?,
            KeyCode::Right => terminal.move_right_for_insert(buffer)?,
            _ => (),
        }
        Ok(())
    }

    fn handle_input_command_mode(
        &mut self,
        buffer: &mut Buffer,
        terminal: &mut Terminal,
        event: KeyEvent,
    ) -> IOResult {
        match event.code {
            KeyCode::Char(char) => {
                self.command_text.insert(self.command_text.len(), char);
                screen::update_command_text(&self.command_text)?;
            }
            KeyCode::Delete => {
                let (x, _y) = Terminal::cursor_position()?;
                if (x) < self.command_text.len() {
                    self.command_text.remove(x);
                    screen::update_command_text(&self.command_text)?;
                }
                if self.command_text.len() == 0 {
                    self.exit_command_mode(buffer, terminal)?;
                }
            }
            KeyCode::Backspace => {
                let (x, _y) = Terminal::cursor_position()?;
                if x as i32 - 1 < 0 {
                    return Ok(());
                }
                queue!(stdout(), MoveLeft(1))?;
                self.command_text.remove(x - 1);
                screen::update_command_text(&self.command_text)?;
                if self.command_text.len() == 0 {
                    self.exit_command_mode(buffer, terminal)?;
                }
            }
            KeyCode::Enter => {
                //EXECUTE
            }
            _ => (),
        }
        Ok(())
    }

    fn exit_command_mode(&mut self, buffer: &Buffer, terminal: &Terminal) -> IOResult {
        self.set_mode(EditorMode::Control)?;
        execute!(stdout(), RestorePosition, SetCursorStyle::BlinkingBlock)?;
        screen::update_line(buffer, terminal, Terminal::size()?.1 as usize)?;
        Ok(())
    }
}
