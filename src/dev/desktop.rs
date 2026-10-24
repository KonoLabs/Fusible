//! Hooks for the desktop shell: a ready line on stdout and a stdin watchdog.

use std::io::Read;
use std::thread;

/// Builds the line the shell waits for once the server is bound.
pub(super) fn ready_line(port: u16) -> String {
    let payload = serde_json::json!({ "url": format!("http://localhost:{port}/"), "port": port });
    format!("FUSIBLE_READY {payload}")
}

/// Prints the ready line, which the shell reads to learn the port.
pub(super) fn announce_ready(port: u16) {
    println!("{}", ready_line(port));
}

/// Blocks until `reader` is at end of file; read errors count as the end.
pub(super) fn wait_for_eof(mut reader: impl Read) {
    let mut sink = [0u8; 256];
    while matches!(reader.read(&mut sink), Ok(read) if read > 0) {}
}

/// Exits the process with code 0 when stdin closes, so a crashed shell leaves no server behind.
pub(super) fn exit_on_stdin_eof() {
    thread::spawn(|| {
        wait_for_eof(std::io::stdin());
        std::process::exit(0);
    });
}
