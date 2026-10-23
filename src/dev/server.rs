use std::sync::mpsc;
use std::thread;

use tiny_http::{Request, Response, Server};

use super::api;
use super::assets::static_response;
use super::bodies;
use super::firmware;
use super::hub::stream_events;
use super::refresh::{rebuild_trigger, refresh, watch_dir};
use super::state::State;
use super::watcher::start_file_watcher;
use crate::commands::target::BoardTarget;

/// Accepts only loopback Host headers bound to the dev server port, blocking DNS rebinding.
pub(super) fn is_allowed_host(host: Option<&str>, port: u16) -> bool {
    host.is_some_and(|h| {
        ["localhost", "127.0.0.1", "[::1]"]
            .iter()
            .any(|name| h.eq_ignore_ascii_case(&format!("{name}:{port}")))
    })
}

/// Accepts a missing Origin, or one that is a loopback page served by this dev server.
pub(super) fn is_allowed_origin(origin: Option<&str>, port: u16) -> bool {
    origin.is_none_or(|o| {
        ["localhost", "127.0.0.1", "[::1]"]
            .iter()
            .any(|name| o.eq_ignore_ascii_case(&format!("http://{name}:{port}")))
    })
}

/// Returns the percent-decoded value of `key` in the URL's query string; empty counts as absent.
pub(super) fn query_param(url: &str, key: &str) -> Option<String> {
    let (_, query) = url.split_once('?')?;
    let value = query
        .split('&')
        .find_map(|pair| pair.strip_prefix(key)?.strip_prefix('='))
        .filter(|value| !value.is_empty())?;
    percent_decode(value)
}

pub(super) fn percent_decode(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = bytes
            .get(i + 1..i + 3)
            .and_then(|h| std::str::from_utf8(h).ok());
        match (bytes[i], hex.and_then(|h| u8::from_str_radix(h, 16).ok())) {
            (b'%', Some(byte)) => (out.push(byte), i += 3),
            (b'+', _) => (out.push(b' '), i += 1),
            (byte, _) => (out.push(byte), i += 1),
        };
    }
    String::from_utf8(out).ok()
}

/// Strips any `?query` from a request URL, leaving the path used for routing.
pub(super) fn route_path(url: &str) -> &str {
    url.split_once('?').map_or(url, |(path, _)| path)
}

fn handle(request: Request, state: &State, port: u16) {
    let host = request.headers().iter().find(|h| h.field.equiv("Host"));
    if !is_allowed_host(host.map(|h| h.value.as_str()), port) {
        let _ = request.respond(Response::from_string("Forbidden").with_status_code(403));
        return;
    }
    let path = route_path(request.url()).to_string();
    match path.as_str() {
        "/api/mechanical" => api::body_preview(request, state, false),
        "/api/mechanical.glb" => api::body_preview(request, state, true),
        "/api/circuit" => api::circuit(request, state),
        "/api/project" => api::project(request, state),
        "/api/runs" => api::runs(request, state),
        p if p.starts_with("/api/bodies/") => bodies::serve(request, p),
        "/api/firmware" => firmware::serve(request, state),
        "/api/run" => api::start_run(request, state, port),
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
    for slot in state.slots.iter() {
        let _ = tx.send(format!("data: {}\n\n", slot.payload()).into_bytes());
    }
    state.hub.add_client(tx);
    thread::spawn(move || stream_events(request, rx));
}

pub fn run_dev_server(
    boards: Vec<BoardTarget>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("127.0.0.1:{port}");
    let server =
        Server::http(&addr).map_err(|e| format!("cannot bind dev server to {addr}: {e}"))?;
    let state = State::new(&boards);
    state.slots.iter().for_each(|slot| refresh(slot, &state));
    if state.bundle.lock().is_ok_and(|bundle| bundle.is_none()) {
        eprintln!(
            "⚠ This fusible build has no studio bundle: schematic, PCB and 3D fall back to native previews. \
Install a GitHub release binary, or run `bun install && bun run build` in studio/ and rebuild."
        );
    }
    println!("⚡ Fusible Dev Server running at http://localhost:{port} (press Ctrl+C to quit)");
    let _watchers = state
        .slots
        .iter()
        .map(|slot| {
            let trigger = rebuild_trigger(slot.clone(), state.clone());
            start_file_watcher(&watch_dir(&slot.entry), trigger)
        })
        .collect::<Result<Vec<_>, _>>()?;
    for request in server.incoming_requests() {
        handle(request, &state, port);
    }
    Ok(())
}
