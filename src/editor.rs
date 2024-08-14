use buffer::Buffer;
use controller::Controller;
use terminal::Terminal;

mod buffer;
mod controller;
mod screen;
mod terminal;

type IOResult = Result<(), std::io::Error>;

pub struct Editor<'a> {
    buffer: Buffer<'a>,
    terminal: Terminal,
    controller: Controller,
}

impl<'a> Default for Editor<'a> {
    fn default() -> Self {
        Editor {
            buffer: Buffer::new(),
            terminal: Terminal::new(),
            controller: Controller::new(),
        }
    }
}

impl<'a> Editor<'a> {
    pub fn new(file_path: Option<&'a String>) -> Self {
        let mut editor = Editor::default();
        if let Err(err) = editor.buffer.read_file(file_path) {
            panic!("{err:#?}")
        }
        editor
    }

    pub fn run(&mut self) {
        if let Err(err) = self.repl() {
            panic!("{err:#?}")
        }
    }

    fn repl(&mut self) -> IOResult {
        Terminal::init()?;
        self.controller.init(&self.buffer, &self.terminal)?;
        self.controller
            .handle_input(&mut self.buffer, &mut self.terminal)?;
        Terminal::clean_up()?;
        Ok(())
    }
}
