use std::io::Cursor;
#[cfg(not(all(studio_bundle, not(debug_assertions))))]
use std::path::PathBuf;

use tiny_http::{Header, Response};

const VIEWER_HTML: &str = include_str!("../assets/viewer.html");
const PROJECT_JS: &str = include_str!("../assets/project.js");
const FIRMWARE_JS: &str = include_str!("../assets/firmware.js");
const DRAWER_JS: &str = include_str!("../assets/drawer.js");
const FONT_DATA: &[u8] = include_bytes!("../assets/undefined-medium.otf");
const FAVICON_SVG: &str = concat!(
    r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">"##,
    r##"<path fill="#86A881" d="M5 3h14v2H5zm0 16h14v2H5zM3 5h2v14H3zm16 0h2v14h-2zM7 7h10v2H7zm0 8h10v2H7z"##,
    r##"m0-6h2v6H7zm8 0h2v6h-2zm-4-8h2v2h-2zm0 20h2v2h-2zM1 11h2v2H1zm20 0h2v2h-2zm0-4h2v2h-2zm0 8h2v2h-2z"##,
    r##"M1 15h2v2H1zm0-8h2v2H1zm6-6h2v2H7zm8 0h2v2h-2zm0 20h2v2h-2zm-8 0h2v2H7z"/></svg>"##,
);
const MISSING_BUNDLE: &str = "// studio.bundle.js not found — run `bun run build` in studio/";

/// A buffered HTTP response as served by the dev server.
pub(super) type HttpResponse = Response<Cursor<Vec<u8>>>;

/// Returns the studio bundle embedded at compile time in release builds.
#[cfg(all(studio_bundle, not(debug_assertions)))]
pub(super) fn load_studio_bundle() -> Option<String> {
    Some(
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/assets/studio.bundle.js"
        ))
        .to_string(),
    )
}

/// Reads the studio bundle from the source tree so a `bun run build` is picked up without recompiling.
#[cfg(not(all(studio_bundle, not(debug_assertions))))]
pub(super) fn load_studio_bundle() -> Option<String> {
    let bundle_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/assets/studio.bundle.js");
    std::fs::read_to_string(&bundle_path).ok()
}

/// Builds a response with `status`, a `Content-Type` header and `body`.
pub(super) fn http_response(status: u16, content_type: &str, body: Vec<u8>) -> HttpResponse {
    let header = Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes())
        .expect("content type is a valid header value");
    Response::from_data(body)
        .with_header(header)
        .with_status_code(status)
}

/// Serves the embedded viewer, font, favicon and studio bundle; `None` for other paths.
pub(super) fn static_response(path: &str, bundle: Option<String>) -> Option<HttpResponse> {
    let response = match path {
        "/" | "/index.html" => http_response(200, "text/html; charset=utf-8", VIEWER_HTML.into()),
        "/studio.bundle.js" => bundle_response(bundle),
        "/project.js" => http_response(
            200,
            "application/javascript; charset=utf-8",
            PROJECT_JS.into(),
        ),
        "/firmware.js" => script_response(FIRMWARE_JS),
        "/drawer.js" => script_response(DRAWER_JS),
        "/undefined-medium.otf" => http_response(200, "font/otf", FONT_DATA.to_vec()),
        "/favicon.ico" | "/favicon.svg" => http_response(200, "image/svg+xml", FAVICON_SVG.into()),
        _ => return None,
    };
    Some(response)
}

fn script_response(source: &str) -> HttpResponse {
    http_response(200, "application/javascript; charset=utf-8", source.into())
}

fn bundle_response(bundle: Option<String>) -> HttpResponse {
    let content_type = "application/javascript; charset=utf-8";
    match bundle {
        Some(content) => http_response(200, content_type, content.into_bytes()),
        None => http_response(404, content_type, MISSING_BUNDLE.into()),
    }
}
