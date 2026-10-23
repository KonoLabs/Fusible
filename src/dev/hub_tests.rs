use std::sync::mpsc::TryRecvError;

use super::hub::ClientHub;

fn text(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes).unwrap()
}

#[test]
fn broadcast_reaches_every_client_and_prunes_the_gone_ones() {
    let hub = ClientHub::default();
    let first = hub.subscribe(Vec::new);
    let second = hub.subscribe(Vec::new);
    hub.broadcast(r#"{"test": true}"#);
    assert_eq!(text(first.recv().unwrap()), "data: {\"test\": true}\n\n");
    assert_eq!(text(second.recv().unwrap()), "data: {\"test\": true}\n\n");
    drop(second);
    hub.broadcast(r#"{"test": 2}"#);
    assert_eq!(text(first.recv().unwrap()), "data: {\"test\": 2}\n\n");
}

#[test]
fn a_client_that_stops_reading_is_dropped_with_a_bounded_queue() {
    let hub = ClientHub::default();
    let slow = hub.subscribe(Vec::new);
    let fast = hub.subscribe(Vec::new);
    for i in 0..500 {
        hub.broadcast(&i.to_string());
        let _ = fast.try_recv();
    }
    let queued = slow.try_iter().count();
    assert!(queued > 0 && queued < 500, "queued {queued}");
    assert!(matches!(slow.try_recv(), Err(TryRecvError::Disconnected)));
    hub.broadcast("last");
    assert_eq!(text(fast.recv().unwrap()), "data: last\n\n");
}

#[test]
fn the_snapshot_arrives_before_any_later_broadcast() {
    let hub = ClientHub::default();
    let rx = hub.subscribe(|| vec!["one".into(), "two".into()]);
    hub.broadcast("three");
    let got: Vec<String> = rx.try_iter().map(text).collect();
    assert_eq!(got, ["data: one\n\n", "data: two\n\n", "data: three\n\n"]);
}
