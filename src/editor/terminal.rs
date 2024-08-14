use crossterm::cursor::{
    position, MoveDown, MoveLeft, MoveRight, MoveTo, MoveToColumn, MoveUp, RestorePosition,
    SavePosition, SetCursorStyle,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, Clear, ClearType, DisableLineWrap, EnableLineWrap,
};
use crossterm::{execute, queue};
use std::io::{stdout, Write};

use super::IOResult;

pub struct Terminal {
    virtual_cursor: (usize, usize),
    virtual_position: (usize, usize),
    virtual_size: (usize, usize),
}

impl Default for Terminal {
    fn default() -> Self {
        Terminal {
            virtual_cursor: (0, 0),
            virtual_position: (0, 0),
            virtual_size: (0, 0),
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

    pub fn clean_up() -> IOResult {
        execute!(stdout(), EnableLineWrap)?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn size() -> Result<(u16, u16), std::io::Error> {
        size()
    }

    pub fn clear_all() -> IOResult {
        execute!(stdout(), Clear(ClearType::All))?;
        Ok(())
    }

    pub fn clear_from_cursor_down() -> IOResult {
        execute!(stdout(), Clear(ClearType::FromCursorDown))?;
        Ok(())
    }

    pub fn clear_line_with_cursor() -> IOResult {
        execute!(stdout(), Clear(ClearType::All))?;
        Ok(())
    }
}
