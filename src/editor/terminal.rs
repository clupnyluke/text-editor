use crossterm::cursor::{
    position, MoveDown, MoveLeft, MoveRight, MoveTo, MoveToColumn, MoveUp, SetCursorStyle,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, Clear, ClearType, DisableLineWrap, EnableLineWrap,
};
use crossterm::{execute, queue};
use std::io::{stdout, Write};

use super::buffer::Buffer;
use super::{screen, IOResult};

pub struct Terminal {
    virtual_cursor: (usize, usize),
    virtual_position: (usize, usize),
}

impl Default for Terminal {
    fn default() -> Self {
        Terminal {
            virtual_cursor: (0, 0),
            virtual_position: (0, 0),
        }
    }
}

impl Terminal {
    pub fn new() -> Self {
        Terminal::default()
    }
    pub fn init() -> IOResult {
        enable_raw_mode()?;
        queue!(
            stdout(),
            Clear(ClearType::All),
            DisableLineWrap,
            SetCursorStyle::BlinkingBlock
        )?;
        stdout().flush()?;
        Ok(())
    }

    pub fn virtual_cursor(&self) -> (usize, usize) {
        self.virtual_cursor
    }

    pub fn virtual_position(&self) -> (usize, usize) {
        self.virtual_position
    }

    pub fn cursor_position() -> Result<(usize, usize), std::io::Error> {
        let (x, y) = position()?;
        Ok((x as usize, y as usize))
    }

    pub fn clean_up() -> IOResult {
        execute!(stdout(), EnableLineWrap)?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn size() -> Result<(usize, usize), std::io::Error> {
        let (x, y) = size()?;
        Ok((x as usize, y as usize))
    }

    pub fn clear_from_cursor_down() -> IOResult {
        execute!(stdout(), Clear(ClearType::FromCursorDown))?;
        Ok(())
    }

    pub fn clear_line_with_cursor() -> IOResult {
        execute!(stdout(), Clear(ClearType::CurrentLine))?;
        Ok(())
    }

    pub fn goto_beginning_of_line(&mut self) -> IOResult {
        execute!(stdout(), MoveToColumn(0))?;
        self.virtual_cursor.0 = 0;
        self.virtual_position.0 = 0;
        Ok(())
    }

    pub fn cursor_to(x: usize, y: usize) -> IOResult {
        execute!(stdout(), MoveTo(x as u16, y as u16))?;
        Ok(())
    }

    pub fn cursor_to_beginning_of_line() -> IOResult {
        execute!(stdout(), MoveToColumn(0))?;
        Ok(())
    }

    pub fn goto(&mut self, buffer: &Buffer, x: usize, y: usize) -> IOResult {
        let (width, height) = Self::size()?;
        let new_cursor_x;
        let new_cursor_y;
        self.virtual_cursor.0 = x;
        let mut rerender = false;
        if x < self.virtual_position.0 {
            self.virtual_position.0 = x;
            new_cursor_x = 0;
            rerender = true;
        } else if x > self.virtual_position.0 + width {
            let diff = x - self.virtual_position.0 - width;
            self.virtual_position.0 += diff;
            new_cursor_x = width;
            rerender = true;
        } else {
            let diff = x - self.virtual_position.0;
            new_cursor_x = diff
        }

        if y < self.virtual_position.1 {
            self.virtual_position.1 = y;
            new_cursor_y = 0;
            rerender = true;
        } else if y > self.virtual_position.1 + height {
            let diff = y - self.virtual_position.1 - height;
            self.virtual_position.1 += diff;
            new_cursor_y = height;
            rerender = true;
        } else {
            let diff = y - self.virtual_position.1;
            new_cursor_y = diff
        }
        if rerender {
            screen::update_line_until_eof(buffer, self, self.virtual_position.1)?;
        }
        Self::cursor_to(new_cursor_x, new_cursor_y)?;
        Ok(())
    }

    pub fn move_down(&mut self, buffer: &Buffer) -> IOResult {
        if self.virtual_cursor.1 >= buffer.len() - 1 {
            return Ok(());
        }
        self.virtual_cursor.1 += 1;
        let default = String::new();
        let (_cursor_x, cursor_y) = Terminal::cursor_position()?;
        let (_width, height) = Terminal::size()?;
        let mut rerender = false;
        if cursor_y == height - 1 {
            self.virtual_position.1 += 1;
            rerender = true;
        } else {
            execute!(stdout(), MoveDown(1))?;
        }
        let line = buffer.get_line(self.virtual_cursor.1).unwrap_or(&default);
        if line.len() < self.virtual_cursor.0 + 1 {
            self.goto(buffer, usize::max(line.len(), 1) - 1, self.virtual_cursor.1)?;
        }

        if rerender {
            screen::update_line_until_eof(buffer, &self, self.virtual_position.1)?;
        }
        Ok(())
    }

    pub fn move_up(&mut self, buffer: &Buffer) -> IOResult {
        if self.virtual_cursor.1 == 0 {
            return Ok(());
        }
        self.virtual_cursor.1 -= 1;
        let default = String::new();
        let (_cursor_x, cursor_y) = Terminal::cursor_position()?;
        let mut rerender = false;
        if cursor_y == 0 {
            self.virtual_position.1 -= 1;
            rerender = true;
        } else {
            execute!(stdout(), MoveUp(1))?;
        }
        let line = buffer.get_line(self.virtual_cursor.1).unwrap_or(&default);
        if line.len() < self.virtual_cursor.0 + 1 {
            self.goto(buffer, usize::max(line.len(), 1) - 1, self.virtual_cursor.1)?;
        } else if rerender {
            screen::update_line_until_eof(buffer, &self, self.virtual_position.1)?;
        }
        Ok(())
    }

    pub fn move_right(&mut self, buffer: &Buffer) -> IOResult {
        self.move_right_base(buffer, false)
    }

    pub fn move_right_for_insert(&mut self, buffer: &Buffer) -> IOResult {
        self.move_right_base(buffer, true)
    }

    fn move_right_base(&mut self, buffer: &Buffer, for_insert_mode: bool) -> IOResult {
        let default = String::new();
        let line = buffer.get_line(self.virtual_cursor.1).unwrap_or(&default);
        let mut line_len = usize::max(line.len(), 0);
        if !for_insert_mode {
            line_len = usize::max(line.len(), 1) - 1;
        }
        if self.virtual_cursor.0 >= line_len {
            return Ok(());
        }
        self.virtual_cursor.0 += 1;
        let (cursor_x, _cursor_y) = Terminal::cursor_position()?;
        let (width, _height) = Terminal::size()?;
        let mut rerender = false;
        if cursor_x == width - 1 {
            self.virtual_position.0 += 1;
            rerender = true;
        } else {
            execute!(stdout(), MoveRight(1))?;
        }
        if rerender {
            screen::update_line_until_eof(buffer, &self, self.virtual_position.1)?;
        }
        Ok(())
    }

    pub fn move_left(&mut self, buffer: &Buffer) -> IOResult {
        if self.virtual_cursor.0 <= 0 {
            return Ok(());
        }
        self.virtual_cursor.0 -= 1;
        let (cursor_x, _cursor_y) = Terminal::cursor_position()?;
        let mut rerender = false;
        if cursor_x == 0 {
            self.virtual_position.0 -= 1;
            rerender = true;
        } else {
            execute!(stdout(), MoveLeft(1))?;
        }
        if rerender {
            screen::update_line_until_eof(buffer, &self, self.virtual_position.1)?;
        }
        Ok(())
    }
}
