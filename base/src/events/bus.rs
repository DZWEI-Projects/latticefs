use crate::error::Result;
use crate::events::types::Event;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::thread;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct EventBus {
    sender: mpsc::Sender<Event>,
}

impl EventBus {
    pub fn new(buffer: usize) -> (Self, mpsc::Receiver<Event>) {
        let (sender, receiver) = mpsc::channel(buffer);
        (Self { sender }, receiver)
    }

    pub async fn emit(&self, event: Event) {
        let _ = self.sender.send(event).await;
    }

    pub fn emit_sync(&self, event: Event) {
        let _ = self.sender.try_send(event);
    }
}

pub fn spawn_logger(mut receiver: mpsc::Receiver<Event>, path: PathBuf) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    thread::spawn(move || {
        let file = OpenOptions::new().create(true).append(true).open(&path);
        if file.is_err() {
            return;
        }
        let file = file.unwrap();
        let mut writer = BufWriter::new(file);

        while let Some(event) = receiver.blocking_recv() {
            if let Ok(line) = serde_json::to_string(&event) {
                let _ = writeln!(writer, "{}", line);
                let _ = writer.flush();
            }
        }
    });

    Ok(())
}
