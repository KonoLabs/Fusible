use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

use tiny_http::{Request, Response, Server};

use super::assets::{http_response, load_studio_bundle, static_response};
use super::hub::{stream_events, ClientHub};
use super::mechanical::MechanicalCache;
use super::pipeline::execute_pipeline;
use super::watcher::start_file_watcher;

#[derive(Clone, Default)]
struct State {
    hub: ClientHub,
    payload: Arc<Mutex<String>>,
    bundle: Arc<Mutex<Option<String>>>,
    mechanical: MechanicalCache,
}

/// Accepts only loopback Host headers bound to the dev server port, blocking DNS rebinding.
pub(super) fn is_allowed_host(host: Option<&str>, port: u16) -> bool {
    host.is_some_and(|h| {
        ["localhost", "127.0.0.1", "[::1]"]
            .iter()
            .any(|name| h.eq_ignore_ascii_case(&format!("{name}:{port}")))
    })
}

/// Strips any `?query` from a request URL, leaving the path used for routing.
pub(super) fn route_path(url: &str) -> &str {
    url.split_once('?').map_or(url, |(path, _)| path)
}

/// Rebuilds the payload, records the circuit source, broadcasts and refreshes the studio bundle.
fn refresh(entry: &Path, state: &State) {
    let start = std::time::Instant::now();
    let (payload, source) = execute_pipeline(entry);
    state.mechanical.update(source, payload.revision.clone());
    if let Ok(json_str) = serde_json::to_string(&payload) {
        if let Ok(mut lock) = state.payload.lock() {
            *lock = json_str.clone();
        }
        state.hub.broadcast(&json_str);
        println!("✓ Circuit updated in {:.1}s", start.elapsed().as_secs_f32());
    }
    if let Ok(mut lock) = state.bundle.lock() {
        *lock = load_studio_bundle();
    }
}

fn watch_dir(entry: &Path) -> PathBuf {
    if entry.is_dir() {
        return entry.to_path_buf();
    }
    entry
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

/// Coalesces file events so at most one rebuild runs at a time and none is lost.
fn rebuild_trigger(entry: PathBuf, state: State) -> impl FnMut() + Send + 'static {
    let rebuilding = Arc::new(AtomicBool::new(false));
    let pending = Arc::new(AtomicBool::new(false));
    move || {
        pending.store(true, Ordering::SeqCst);
        if rebuilding.swap(true, Ordering::SeqCst) {
            return;
        }
        let (entry, state) = (entry.clone(), state.clone());
        let (rebuilding, pending) = (rebuilding.clone(), pending.clone());
        thread::spawn(move || loop {
            while pending.swap(false, Ordering::SeqCst) {
                println!("🔄 File change detected — rebuilding circuit preview...");
                refresh(&entry, &state);
            }
            rebuilding.store(false, Ordering::SeqCst);
            if !pending.load(Ordering::SeqCst) || rebuilding.swap(true, Ordering::SeqCst) {
                break;
            }
        });
    }
}

fn handle(request: Request, state: &State, port: u16) {
    let host = request.headers().iter().find(|h| h.field.equiv("Host"));
    if !is_allowed_host(host.map(|h| h.value.as_str()), port) {
        let _ = request.respond(Response::from_string("Forbidden").with_status_code(403));
        return;
    }
    let path = route_path(request.url()).to_string();
    match path.as_str() {
        "/api/mechanical" => state.mechanical.serve(request, false),
        "/api/mechanical.glb" => state.mechanical.serve(request, true),
        "/api/circuit" => {
            let body = state
                .payload
                .lock()
                .map(|l| l.clone())
                .unwrap_or_else(|_| "{}".into());
            let _ = request.respond(http_response(200, "application/json", body.into_bytes()));
        }
        "/events" => subscribe(request, state),
        _ => {
            let bundle = state.bundle.lock().ok().and_then(|lock| lock.clone());
            let response = static_response(&path, bundle)
                .unwrap_or_else(|| Response::from_string("Not Found").with_status_code(404));
            let _ = request.respond(response);
        }
    }
}

fn subscribe(request: Request, state: &State) {
    let (tx, rx) = mpsc::channel();
    if let Ok(lock) = state.payload.lock() {
        let _ = tx.send(format!("data: {}\n\n", *lock).into_bytes());
    }
    state.hub.add_client(tx);
    thread::spawn(move || stream_events(request, rx));
}

pub fn run_dev_server(entry: PathBuf, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("127.0.0.1:{port}");
    let server =
        Server::http(&addr).map_err(|e| format!("cannot bind dev server to {addr}: {e}"))?;
    let state = State::default();
    refresh(&entry, &state);
    if state.bundle.lock().is_ok_and(|bundle| bundle.is_none()) {
        eprintln!(
            "⚠ This fusible build has no studio bundle: schematic, PCB and 3D fall back to native previews. \
Install a GitHub release binary, or run `bun install && bun run build` in studio/ and rebuild."
        );
    }
    println!("⚡ Fusible Dev Server running at http://localhost:{port} (press Ctrl+C to quit)");
    let _watcher = start_file_watcher(
        &watch_dir(&entry),
        rebuild_trigger(entry.clone(), state.clone()),
    )?;
    for request in server.incoming_requests() {
        handle(request, &state, port);
    }
    Ok(())
}
