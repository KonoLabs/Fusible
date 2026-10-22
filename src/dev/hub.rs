use std::io::Write;
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const SSE_HEARTBEAT: Duration = Duration::from_secs(15);

#[derive(Clone, Default)]
pub struct ClientHub {
    clients: Arc<Mutex<Vec<Sender<Vec<u8>>>>>,
}

impl ClientHub {
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

/// Streams SSE messages to one client, sending a heartbeat so dead sockets are detected.
pub fn stream_events(request: tiny_http::Request, rx: mpsc::Receiver<Vec<u8>>) {
    let mut writer = request.into_writer();
    let headers = "HTTP/1.1 200 OK\r\n\
Content-Type: text/event-stream\r\n\
Cache-Control: no-cache\r\n\
Connection: keep-alive\r\n\
\r\n";
    if writer.write_all(headers.as_bytes()).is_err() || writer.flush().is_err() {
        return;
    }
    loop {
        let msg = match rx.recv_timeout(SSE_HEARTBEAT) {
            Ok(msg) => msg,
            Err(RecvTimeoutError::Timeout) => b": ping\n\n".to_vec(),
            Err(RecvTimeoutError::Disconnected) => break,
        };
        if writer.write_all(&msg).is_err() || writer.flush().is_err() {
            break;
        }
    }
}
