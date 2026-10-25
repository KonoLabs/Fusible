use std::io::{Read, Write};
use std::net::TcpStream;

use tiny_http::Server;

use super::server::handle;
use super::state::State;

fn exchange(method: &str, target: &str, extra: &str) -> String {
    let server = Server::http("127.0.0.1:0").unwrap();
    let port = server.server_addr().to_ip().unwrap().port();
    let state = State::new(&[]);
    let worker = std::thread::spawn(move || handle(server.recv().unwrap(), &state, port));
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    let head = format!(
        "{method} {target} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n{extra}Connection: close\r\n\r\n"
    );
    stream.write_all(head.as_bytes()).unwrap();
    let mut text = String::new();
    stream.read_to_string(&mut text).unwrap();
    worker.join().unwrap();
    text
}

fn status(text: &str) -> &str {
    text.lines().next().unwrap()
}

#[test]
fn requests_reach_the_assembly_routes_through_the_server() {
    assert!(status(&exchange("GET", "/api/assembly/model?rev=7", "")).contains("404"));
    assert!(status(&exchange("GET", "/api/assembly", "")).contains("200"));
    assert!(status(&exchange("POST", "/api/assembly", "Content-Length: 0\r\n")).contains("405"));
}

#[test]
fn the_same_origin_guard_covers_the_assembly_routes() {
    let foreign = "Sec-Fetch-Site: cross-site\r\n";
    for path in ["/api/assembly", "/api/assembly/model?rev=1"] {
        assert!(status(&exchange("GET", path, foreign)).contains("403"));
    }
}
