use std::io::{self, Read};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct ClientHub {
    clients: Arc<Mutex<Vec<Sender<Vec<u8>>>>>,
}

impl ClientHub {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_client(&self, tx: Sender<Vec<u8>>) {
        if let Ok(mut lock) = self.clients.lock() {
            lock.push(tx);
        }
    }

    pub fn broadcast(&self, payload: &str) {
        let msg = format!("data: {}\n\n", payload).into_bytes();
        if let Ok(mut lock) = self.clients.lock() {
            lock.retain(|tx| tx.send(msg.clone()).is_ok());
        }
    }
}

pub struct SseReader {
    rx: Receiver<Vec<u8>>,
    buffer: Vec<u8>,
}

impl SseReader {
    pub fn new(rx: Receiver<Vec<u8>>) -> Self {
        Self {
            rx,
            buffer: Vec::new(),
        }
    }
}

impl Read for SseReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.buffer.is_empty() {
            match self.rx.recv() {
                Ok(data) => self.buffer = data,
                Err(_) => return Ok(0),
            }
        }
        let count = std::cmp::min(buf.len(), self.buffer.len());
        buf[..count].copy_from_slice(&self.buffer[..count]);
        self.buffer.drain(..count);
        Ok(count)
    }
}
