use tiny_http::{Request, Response};

use super::assets::static_response;
use super::state::State;

pub(super) fn serve_static(request: Request, state: &State, path: &str) {
    let bundle = state.bundle.lock().ok().and_then(|lock| lock.clone());
    let response = static_response(path, bundle, state.desktop)
        .unwrap_or_else(|| Response::from_string("Not Found").with_status_code(404));
    let _ = request.respond(response);
}
