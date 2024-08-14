use std::io::{stdout, Write};

use crossterm::cursor::{position, MoveTo, MoveToColumn, RestorePosition, SavePosition};
use crossterm::terminal::{Clear, ClearType};
use crossterm::{execute, queue};

use super::buffer::Buffer;
use super::IOResult;

pub fn update_line(buffer: &Buffer, line_number: u16) -> IOResult {
    queue!(
        stdout(),
        SavePosition,
        Clear(ClearType::CurrentLine),
        MoveTo(0, line_number)
    )?;
    let default = String::new();
    let line = buffer.get_line(line_number).unwrap_or(&default);
    print!("{line}");
    queue!(stdout(), RestorePosition)?;
    stdout().flush()?;
    Ok(())
}

pub fn update_current_line(buffer: &Buffer) -> IOResult {
    update_line(buffer, position()?.1)
}

pub fn update_line_until_eof(buffer: &Buffer, line_number: u16) -> IOResult {
    queue!(
        stdout(),
        SavePosition,
        MoveTo(0, line_number),
        Clear(ClearType::CurrentLine),
        Clear(ClearType::FromCursorDown)
    )?;
    let default = vec![String::new()];
    let lines = buffer.get_lines(line_number as usize..).unwrap_or(&default);
    for line in lines {
        println!("{line}");
        queue!(stdout(), MoveToColumn(0))?;
    }
    queue!(stdout(), RestorePosition)?;
    stdout().flush()?;
    Ok(())
}
