use std::fs;

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

    fn write_to_file(&self, file_path: &String) -> IOResult {
        fs::write(file_path, self.contents.join("\n"))?;
        Ok(())
    }

    pub fn write_file(&self, file_path: Option<&String>) -> IOResult {
        if let Some(path) = file_path {
            self.write_to_file(path)?;
        } else if let Some(path) = self.file_path {
            self.write_to_file(path)?;
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "No file to write to",
            ));
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
        if row > self.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Row doesn't exist",
            ));
        }
        let line = self.get_line_mut(row).unwrap();
        if column > line.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Column out of Bounds",
            ));
        }
        (*line).insert(column as usize, char);
        terminal.move_right_for_insert(&self)?;
        screen::update_line(self, terminal, row)?;
        Ok(())
    }

    pub fn delete_char_on_line(
        &mut self,
        terminal: &Terminal,
        row: usize,
        column: usize,
    ) -> IOResult {
        if row > self.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Row doesn't exist",
            ));
        }
        let line = self.get_line_mut(row).unwrap();
        if column > line.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Column out of Bounds",
            ));
        }
        (*line).remove(column as usize);
        screen::update_current_line(&self, terminal)?;
        Ok(())
    }

    pub fn delete_line(&mut self, row: usize) -> IOResult {
        if row > self.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Row doesn't exist",
            ));
        }
        self.contents.splice(row..row + 1, []).for_each(drop);
        Ok(())
    }

    pub fn move_line_contents_up_one_row(&mut self, row: usize) -> IOResult {
        if row > self.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Row doesn't exist",
            ));
        }
        if row == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Top Row can't be moved up",
            ));
        }
        let line = self.get_line(row).unwrap();
        let append_str = line.clone();
        let line_upper = self.get_line_mut(i - 1).unwrap();
        (*line_upper).push_str(append_str.as_str());
    }

    pub fn insert_line(&mut self, row: usize, contents: String) -> IOResult {
        if row > self.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Out of Bounds",
            ));
        }
        self.contents.insert(row as usize, contents);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.contents.len()
    }
}
