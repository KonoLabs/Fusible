use std::io::Write;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const SSE_HEARTBEAT: Duration = Duration::from_secs(15);
const QUEUE: usize = 64;

#[derive(Clone, Default)]
pub struct ClientHub {
    clients: Arc<Mutex<Vec<SyncSender<Vec<u8>>>>>,
}

fn message(payload: &str) -> Vec<u8> {
    format!("data: {payload}\n\n").into_bytes()
}

impl ClientHub {
    /// Registers a client and queues `snapshot()` for it, atomically with respect to broadcasts.
    ///
    /// The snapshot is read while the client list is locked, so a broadcast can neither be
    /// missed nor arrive before the snapshot it supersedes.
    pub fn subscribe(&self, snapshot: impl FnOnce() -> Vec<String>) -> Receiver<Vec<u8>> {
        let (tx, rx) = mpsc::sync_channel(QUEUE);
        if let Ok(mut lock) = self.clients.lock() {
            let delivered = snapshot().iter().all(|p| tx.try_send(message(p)).is_ok());
            if delivered {
                lock.push(tx);
            }
        }
        rx
    }

    /// Sends `payload` to every client and drops the ones that are gone or fell behind.
    pub fn broadcast(&self, payload: &str) {
        let msg = message(payload);
        if let Ok(mut lock) = self.clients.lock() {
            lock.retain(|tx| {
                !matches!(
                    tx.try_send(msg.clone()),
                    Err(TrySendError::Full(_) | TrySendError::Disconnected(_))
                )
            });
        }
    }
}

/// Streams SSE messages to one client, sending a heartbeat so dead sockets are detected.
pub fn stream_events(request: tiny_http::Request, rx: Receiver<Vec<u8>>) {
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
