#![warn(clippy::all, clippy::pedantic)]
use editor::Editor;

pub mod editor;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut path: Option<&String> = None;
    if args.len() > 1 {
        path = Some(&args[1]);
    }
    Editor::new(path).run();
}
