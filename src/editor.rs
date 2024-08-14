use buffer::Buffer;
use controller::Controller;

mod buffer;
mod controller;
mod screen;
mod terminal;

type IOResult = Result<(), std::io::Error>;

pub struct Editor<'a> {
    buffer: Buffer<'a>,
    controller: Controller,
}

impl<'a> Default for Editor<'a> {
    fn default() -> Self {
        Editor {
            buffer: Buffer::new(),
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
        self.controller.init(&self.buffer)?;
        self.controller.handle_input(&mut self.buffer)?;
        self.controller.terminate()?;
        Ok(())
    }
}
