//! Integration tests for `fusible dev --desktop`: the ready line, the viewer flag, and the stdin watchdog.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn fusible() -> Command {
    Command::new(env!("CARGO_BIN_EXE_fusible"))
}

fn free_port() -> u16 {
    TcpListener::bind(("127.0.0.1", 0))
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn get(port: u16) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    write!(
        stream,
        "GET / HTTP/1.1\r\nHost: localhost:{port}\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut body = String::new();
    stream.read_to_string(&mut body).unwrap();
    body
}

#[test]
fn desktop_dev_prints_the_ready_line_serves_the_flag_and_exits_when_stdin_closes() {
    let port = free_port();
    let mut child = fusible()
        .args(["dev", "--desktop", "-p", &port.to_string()])
        .arg("examples/circuit.json")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("binary should run");
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let ready = lines
        .find_map(|line| line.ok()?.strip_prefix("FUSIBLE_READY ").map(String::from))
        .expect("a ready line");
    let payload: serde_json::Value = serde_json::from_str(&ready).unwrap();
    assert_eq!(payload["port"], port);
    assert_eq!(payload["url"], format!("http://localhost:{port}/"));

    assert!(get(port).contains("window.FUSIBLE_DESKTOP = true;"));

    drop(child.stdin.take());
    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            panic!("fusible kept running after stdin closed");
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    assert_eq!(status.code(), Some(0));
}

#[test]
fn plain_dev_serves_no_desktop_flag() {
    let port = free_port();
    let mut child = fusible()
        .args(["dev", "-p", &port.to_string()])
        .arg("examples/circuit.json")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("binary should run");
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    assert!(lines.any(|line| line.is_ok_and(|l| l.contains("Dev Server running"))));
    let page = get(port);
    let _ = child.kill();
    let _ = child.wait();
    assert!(!page.contains("FUSIBLE_DESKTOP"));
}
