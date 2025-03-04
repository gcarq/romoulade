use std::io;
use std::sync::mpsc;
use std::thread;

use termion::event::Key;
use termion::input::TermRead;

pub enum Event<I> {
    Input(I),
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
    #[allow(unused)]
    input_handle: thread::JoinHandle<()>,
}

impl Events {
    pub fn new() -> Events {
        let (tx, rx) = mpsc::channel();
        let input_handle = {
            thread::spawn(move || {
                let stdin = io::stdin();
                for key in stdin.keys().flatten() {
                    if let Err(err) = tx.send(Event::Input(key)) {
                        eprintln!("{}", err);
                        return;
                    }
                }
            })
        };
        Events { rx, input_handle }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }
}
