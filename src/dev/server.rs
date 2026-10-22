use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

use tiny_http::{Header, Response, Server};

use super::hub::ClientHub;
use super::pipeline::execute_pipeline;
use super::watcher::start_file_watcher;

const VIEWER_HTML: &str = include_str!("../assets/viewer.html");
const FONT_DATA: &[u8] = include_bytes!("../assets/undefined-medium.otf");

fn load_studio_bundle() -> Option<String> {
    let bundle_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/assets/studio.bundle.js");
    std::fs::read_to_string(&bundle_path).ok()
}

pub fn run_dev_server(entry: PathBuf, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("127.0.0.1:{port}");
    let server =
        Server::http(&addr).map_err(|e| format!("cannot bind dev server to {addr}: {e}"))?;

    let hub = ClientHub::new();
    let initial_payload = execute_pipeline(&entry);
    let cached_payload = Arc::new(Mutex::new(serde_json::to_string(&initial_payload)?));
    let cached_bundle = Arc::new(Mutex::new(load_studio_bundle()));

    println!("⚡ Fusible Dev Server running at http://localhost:{port} (press Ctrl+C to quit)");

    let watch_dir = if entry.is_dir() {
        entry.clone()
    } else {
        entry
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    };

    let hub_clone = hub.clone();
    let entry_clone = entry.clone();
    let cache_clone = cached_payload.clone();
    let bundle_clone = cached_bundle.clone();
    let is_rebuilding = Arc::new(AtomicBool::new(false));

    let is_rebuilding_clone = is_rebuilding.clone();
    let _watcher = start_file_watcher(&watch_dir, move || {
        if is_rebuilding_clone.swap(true, Ordering::SeqCst) {
            return;
        }
        let hub = hub_clone.clone();
        let entry = entry_clone.clone();
        let cache = cache_clone.clone();
        let bundle = bundle_clone.clone();
        let flag = is_rebuilding_clone.clone();
        thread::spawn(move || {
            println!("🔄 File change detected — rebuilding circuit preview...");
            let start = std::time::Instant::now();
            let payload = execute_pipeline(&entry);
            if let Ok(json_str) = serde_json::to_string(&payload) {
                if let Ok(mut lock) = cache.lock() {
                    *lock = json_str.clone();
                }
                hub.broadcast(&json_str);
                println!("✓ Circuit updated in {:.1}s", start.elapsed().as_secs_f32());
            }
            if let Some(bundle_content) = load_studio_bundle() {
                if let Ok(mut lock) = bundle.lock() {
                    *lock = Some(bundle_content);
                }
            }
            flag.store(false, Ordering::SeqCst);
        });
    })?;

    for request in server.incoming_requests() {
        match request.url() {
            "/" | "/index.html" => {
                let header =
                    Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
                        .unwrap();
                let response = Response::from_string(VIEWER_HTML).with_header(header);
                let _ = request.respond(response);
            }
            "/studio.bundle.js" => {
                let header = Header::from_bytes(
                    &b"Content-Type"[..],
                    &b"application/javascript; charset=utf-8"[..],
                )
                .unwrap();
                if let Ok(lock) = cached_bundle.lock() {
                    if let Some(ref content) = *lock {
                        let response = Response::from_string(content.as_str()).with_header(header);
                        let _ = request.respond(response);
                    } else {
                        let response = Response::from_string(
                            "// studio.bundle.js not found — run `bun run build` in studio/",
                        )
                        .with_header(header)
                        .with_status_code(404);
                        let _ = request.respond(response);
                    }
                } else {
                    let response = Response::from_string(
                        "// studio.bundle.js not found — run `bun run build` in studio/",
                    )
                    .with_header(header)
                    .with_status_code(404);
                    let _ = request.respond(response);
                }
            }
            "/undefined-medium.otf" => {
                let header = Header::from_bytes(&b"Content-Type"[..], &b"font/otf"[..]).unwrap();
                let response = Response::from_data(FONT_DATA).with_header(header);
                let _ = request.respond(response);
            }
            "/favicon.ico" | "/favicon.svg" => {
                let header =
                    Header::from_bytes(&b"Content-Type"[..], &b"image/svg+xml"[..]).unwrap();
                let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24"><path fill="#86A881" d="M5 3h14v2H5zm0 16h14v2H5zM3 5h2v14H3zm16 0h2v14h-2zM7 7h10v2H7zm0 8h10v2H7zm0-6h2v6H7zm8 0h2v6h-2zm-4-8h2v2h-2zm0 20h2v2h-2zM1 11h2v2H1zm20 0h2v2h-2zm0-4h2v2h-2zm0 8h2v2h-2zM1 15h2v2H1zm0-8h2v2H1zm6-6h2v2H7zm8 0h2v2h-2zm0 20h2v2h-2zm-8 0h2v2H7z"/></svg>"##;
                let response = Response::from_string(svg).with_header(header);
                let _ = request.respond(response);
            }
            "/api/circuit" => {
                let header =
                    Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
                let body = cached_payload
                    .lock()
                    .map(|l| l.clone())
                    .unwrap_or_else(|_| "{}".to_string());
                let response = Response::from_string(body).with_header(header);
                let _ = request.respond(response);
            }
            "/events" => {
                let (tx, rx) = mpsc::channel();
                if let Ok(lock) = cached_payload.lock() {
                    let initial_msg = format!("data: {}\n\n", *lock).into_bytes();
                    let _ = tx.send(initial_msg);
                }
                hub.add_client(tx);

                thread::spawn(move || {
                    let mut writer = request.into_writer();
                    let headers = "HTTP/1.1 200 OK\r\n\
Content-Type: text/event-stream\r\n\
Cache-Control: no-cache\r\n\
Connection: keep-alive\r\n\
Access-Control-Allow-Origin: *\r\n\
\r\n";
                    if writer.write_all(headers.as_bytes()).is_err() || writer.flush().is_err() {
                        return;
                    }

                    while let Ok(msg) = rx.recv() {
                        if writer.write_all(&msg).is_err() || writer.flush().is_err() {
                            break;
                        }
                    }
                });
            }
            _ => {
                let response = Response::from_string("Not Found").with_status_code(404);
                let _ = request.respond(response);
            }
        }
    }

    Ok(())
}
