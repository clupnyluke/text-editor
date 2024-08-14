use crossterm::cursor::{
    position, MoveDown, MoveLeft, MoveRight, MoveTo, MoveUp, RestorePosition, SavePosition,
    SetCursorStyle,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, DisableLineWrap, EnableLineWrap,
};

use std::io::{stdout, Read, Stdout, Write};
use std::ops::{Range, RangeFrom};
use std::rc::Rc;
use std::slice::SliceIndex;

use crossterm::QueueableCommand;

use super::{screen, IOResult};

pub struct Buffer<'a> {
    contents: Vec<String>,
    file_path: Option<&'a String>,
}

impl<'a> Default for Buffer<'a> {
    fn default() -> Self {
        Buffer {
            contents: vec![String::new()],
            file_path: None,
        }
    }
}

impl<'a> Buffer<'a> {
    pub fn new() -> Self {
        Buffer::default()
    }

    pub fn read_file(&mut self, file: Option<&'a String>) -> IOResult {
        self.file_path = file;
        if let Some(path) = file {
            let mut contents = String::new();
            let mut file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(path)?;
            file.read_to_string(&mut contents)?;
            self.contents = contents.lines().map(|val| String::from(val)).collect();
        }
        Ok(())
    }

    pub fn get_contents(&self) -> &Vec<String> {
        &self.contents
    }

    pub fn get_line(&self, i: u16) -> Option<&String> {
        self.contents.get(i as usize)
    }

    pub fn get_line_mut(&mut self, i: u16) -> Option<&mut String> {
        self.contents.get_mut(i as usize)
    }

    pub fn insert_char_on_line(&mut self, char: char, row: u16, column: u16) -> IOResult {
        let line = self.get_line_mut(row).unwrap();
        (*line).insert(column as usize, char);
        stdout().queue(MoveRight(1))?;
        screen::update_line(self, row)?;
        Ok(())
    }

    pub fn delete_char_on_line(&mut self, row: u16, column: u16) -> IOResult {
        let len = self.len();
        let line = self.get_line_mut(row).unwrap();
        (*line).remove(column as usize);
        screen::update_current_line(&self)?;
        Ok(())
    }

    pub fn get_lines(&self, range: RangeFrom<usize>) -> Option<&[String]> {
        self.contents.get(range)
    }

    pub fn get_lines_mut(&mut self, range: RangeFrom<usize>) -> Option<&mut [String]> {
        self.contents.get_mut(range)
    }

    pub fn delete_line(&mut self, i: u16) {
        self.contents
            .splice(i as usize..i as usize + 1, [])
            .for_each(drop);
    }

    pub fn move_line_contents_up_one_row(&mut self, i: u16) {
        let line = self.get_line(i).unwrap();
        let append_str = line.clone();
        let line_upper = self.get_line_mut(i - 1).unwrap();
        (*line_upper).push_str(append_str.as_str());
    }

    pub fn insert_line(&mut self, i: u16, contents: String) {
        self.contents.insert(i as usize, contents);
    }

    pub fn len(&self) -> usize {
        self.contents.len()
    }
}
