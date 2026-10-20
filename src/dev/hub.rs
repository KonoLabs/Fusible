use std::sync::mpsc::Sender;
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
