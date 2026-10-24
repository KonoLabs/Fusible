use std::net::TcpListener;

use super::bind::bind;

#[test]
fn a_taken_port_moves_to_the_next_free_one() {
    let taken = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = taken.local_addr().unwrap().port();
    let (_server, got) = bind(port).unwrap();
    assert!(got > port);
}
