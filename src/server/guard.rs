use std::io::{self, Write};
use std::sync::mpsc::{SyncSender, SendError};

pub struct Guard(SyncSender<()>);

impl Guard {
    pub fn new(tx: SyncSender<()>) -> Self {
        Guard(tx)
    }

    pub fn stop(self) -> Result<(), SendError<()>> {
        self.0.send(())
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        if let Err(e) = self.0.send(()) {
            writeln!(io::stderr(), "Error stopping server thread: {}", e)
                .expect("Unable to write to stderr");
        }
    }
}
