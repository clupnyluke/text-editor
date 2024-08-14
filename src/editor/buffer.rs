use crossterm::cursor::MoveRight;

use std::io::{stdout, Read};

use crossterm::QueueableCommand;

use super::terminal::Terminal;
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

    pub fn get_line(&self, i: usize) -> Option<&String> {
        self.contents.get(i)
    }

    pub fn get_line_mut(&mut self, i: usize) -> Option<&mut String> {
        self.contents.get_mut(i)
    }

    pub fn insert_char_on_line(
        &mut self,
        terminal: &mut Terminal,
        char: char,
        row: usize,
        column: usize,
    ) -> IOResult {
        let line = self.get_line_mut(row).unwrap();
        (*line).insert(column as usize, char);
        stdout().queue(MoveRight(1))?;
        screen::update_line(self, terminal, row)?;
        Ok(())
    }

    pub fn delete_char_on_line(
        &mut self,
        terminal: &Terminal,
        row: usize,
        column: usize,
    ) -> IOResult {
        let line = self.get_line_mut(row).unwrap();
        (*line).remove(column as usize);
        screen::update_current_line(&self, terminal)?;
        Ok(())
    }

    pub fn delete_line(&mut self, i: usize) {
        self.contents
            .splice(i as usize..i as usize + 1, [])
            .for_each(drop);
    }

    pub fn move_line_contents_up_one_row(&mut self, i: usize) {
        let line = self.get_line(i).unwrap();
        let append_str = line.clone();
        let line_upper = self.get_line_mut(i - 1).unwrap();
        (*line_upper).push_str(append_str.as_str());
    }

    pub fn insert_line(&mut self, i: usize, contents: String) {
        self.contents.insert(i as usize, contents);
    }

    pub fn len(&self) -> usize {
        self.contents.len()
    }
}
