use std::io::{stdout, Write};

use crossterm::cursor::{position, MoveTo, MoveToColumn, RestorePosition, SavePosition};
use crossterm::{execute, queue};

use super::buffer::Buffer;
use super::terminal::Terminal;
use super::IOResult;

pub fn update_line(buffer: &Buffer, line_number: u16) -> IOResult {
    queue!(stdout(), SavePosition, MoveTo(0, line_number))?;
    Terminal::clear_line_with_cursor()?;
    let default = String::new();
    if (line_number as usize) < buffer.len() {
        let line = buffer.get_line(line_number).unwrap_or(&default);
        print!("{line}");
    } else {
        print!("~");
    }
    queue!(stdout(), RestorePosition)?;
    stdout().flush()?;
    Ok(())
}

pub fn update_current_line(buffer: &Buffer) -> IOResult {
    update_line(buffer, position()?.1)
}

pub fn update_line_until_eof(buffer: &Buffer, line_number: u16) -> IOResult {
    queue!(stdout(), SavePosition, MoveTo(0, line_number),)?;
    Terminal::clear_from_cursor_down()?;
    let default = vec![String::new()];
    let lines = buffer.get_lines(line_number as usize..).unwrap_or(&default);
    let mut line_number = 0;
    for line in lines {
        println!("{line}");
        queue!(stdout(), MoveToColumn(0))?;
        line_number += 1;
    }
    for _ in line_number..=Terminal::size()?.1 {
        println!("~");
        queue!(stdout(), MoveToColumn(0))?;
    }

    queue!(stdout(), RestorePosition)?;
    stdout().flush()?;
    Ok(())
}

pub fn update_command_text(command_text: &String) -> IOResult {
    let (_, eof) = Terminal::size()?;
    queue!(stdout(), MoveTo(0, eof))?;
    Terminal::clear_from_cursor_down()?;
    print!("{command_text}");
    stdout().flush()?;
    Ok(())
}
