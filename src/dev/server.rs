use std::thread;

use tiny_http::{Request, Response};

use super::api;
use super::assets::static_response;
use super::bind::bind;
use super::bodies;
use super::bom;
use super::firmware;
use super::guard::{is_allowed_host, is_same_origin};
use super::hub::stream_events;
use super::kicad_watch::start_kicad_watchers;
use super::limiter::handlers;
use super::refresh::{load_bundle, rebuild_trigger, refresh, watch_dir};
use super::run_api;
use super::state::{Slot, State};
use super::watcher::start_file_watcher;
use crate::commands::target::BoardTarget;

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
    let guarded = path.starts_with("/api/") || path == "/events";
    if guarded && !is_same_origin(&request, port) {
        let _ = request.respond(Response::from_string("Forbidden").with_status_code(403));
        return;
    }
    match path.as_str() {
        "/api/mechanical" => api::body_preview(request, state, false),
        "/api/mechanical.glb" => api::body_preview(request, state, true),
        "/api/circuit" => api::circuit(request, state),
        "/api/project" => api::project(request, state),
        "/api/bom" => bom::serve(request, state, port),
        "/api/runs" => api::runs(request, state),
        p if p.starts_with("/api/bodies/") => bodies::serve(request, p),
        "/api/kicad" => api::kicad(request, state),
        "/api/firmware" => firmware::serve(request, state),
        "/api/run" => run_api::start_run(request, state, port),
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
    let ready = || {
        state
            .slots
            .iter()
            .map(Slot::payload)
            .filter(|p| !p.is_empty())
    };
    let rx = state.hub.subscribe(|| ready().collect());
    thread::spawn(move || stream_events(request, rx));
}

/// Handles each request on its own thread so a stalled client cannot block the others.
///
/// At most [`limiter::handlers`] requests run at once; the rest get a 503.
fn dispatch(request: Request, state: &State, port: u16) {
    let Some(permit) = handlers().try_acquire() else {
        let _ = request.respond(Response::from_string("Busy").with_status_code(503));
        return;
    };
    let state = state.clone();
    thread::spawn(move || {
        handle(request, &state, port);
        drop(permit);
    });
}

/// Builds every board in the background, so the server answers while compiles run.
fn refresh_all(state: &State) {
    for slot in state.slots.iter() {
        let (slot, state) = (slot.clone(), state.clone());
        thread::spawn(move || refresh(&slot, &state));
    }
}

pub fn run_dev_server(
    boards: Vec<BoardTarget>,
    asked: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let (server, port) = bind(asked)?;
    if port != asked {
        println!("ℹ Port {asked} is in use, using {port} instead.");
    }
    let state = State::new(&boards);
    load_bundle(&state);
    if state.bundle.lock().is_ok_and(|bundle| bundle.is_none()) {
        eprintln!(
            "⚠ This fusible build has no studio bundle: schematic, PCB and 3D fall back to native previews. \
Install a GitHub release binary, or run `bun install && bun run build` in studio/ and rebuild."
        );
    }
    refresh_all(&state);
    println!("⚡ Fusible Dev Server running at http://localhost:{port} (press Ctrl+C to quit)");
    let _watchers = state
        .slots
        .iter()
        .map(|slot| {
            let trigger = rebuild_trigger(slot.clone(), state.clone());
            start_file_watcher(&watch_dir(&slot.entry), trigger)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let _kicad = start_kicad_watchers(&state);
    for request in server.incoming_requests() {
        dispatch(request, &state, port);
    }
    Ok(())
}
