use std::io::{stdout, Write};

use crossterm::cursor::{MoveTo, RestorePosition, SavePosition};
use crossterm::queue;

use super::buffer::Buffer;
use super::terminal::Terminal;
use super::IOResult;

pub fn update_line(buffer: &Buffer, terminal: &Terminal, line_number: usize) -> IOResult {
    let (term_x, _term_y) = terminal.virtual_position();
    let (width, _height) = Terminal::size()?;
    queue!(stdout(), SavePosition,)?;
    Terminal::cursor_to_beginning_of_line()?;
    Terminal::clear_line_with_cursor()?;
    let default = String::new();
    if (line_number as usize) < buffer.len() {
        let line = buffer.get_line(line_number).unwrap_or(&default);
        let text = &line[term_x..usize::min(term_x + width as usize, line.len())];
        print!("{text}");
    } else {
        print!("~");
    }
    queue!(stdout(), RestorePosition)?;
    stdout().flush()?;
    Ok(())
}

pub fn update_current_line(buffer: &Buffer, terminal: &Terminal) -> IOResult {
    update_line(buffer, terminal, Terminal::cursor_position()?.1)
}

pub fn update_line_until_eof(buffer: &Buffer, terminal: &Terminal, line_number: usize) -> IOResult {
    let (term_x, term_y) = terminal.virtual_position();
    let (width, height) = Terminal::size()?;
    if line_number >= term_y && line_number <= term_y + height as usize {
        queue!(stdout(), SavePosition)?;
        Terminal::cursor_to(0, line_number - term_y)?;
        Terminal::clear_from_cursor_down()?;
        let default = String::from("~");
        for line_number in line_number..term_y + height {
            let line = buffer.get_line(line_number).unwrap_or(&default);
            let mut text = "";
            if term_x <= line.len() {
                text = &line[term_x..usize::min(term_x + width as usize, line.len())];
            }
            if line_number == term_y + height - 1 {
                print!("{text}");
            } else {
                println!("{text}");
            }
            Terminal::cursor_to_beginning_of_line()?;
        }
        queue!(stdout(), RestorePosition)?;
        stdout().flush()?;
    }
    Ok(())
}

pub fn update_command_text(command_text: &String) -> IOResult {
    let (_, eof) = Terminal::size()?;
    queue!(stdout(), MoveTo(0, eof as u16))?;
    Terminal::clear_line_with_cursor()?;
    print!("{command_text}");
    stdout().flush()?;
    Ok(())
}
