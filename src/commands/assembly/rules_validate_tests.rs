use super::rules::{Measurement, Response};
use super::rules_support::{board_probes, hit};
use super::rules_validate::validate;

fn answered(hits: Vec<Measurement>) -> Response {
    Response {
        version: 1,
        measured: board_probes().into_iter().map(|p| p.id).collect(),
        hits,
        ..Default::default()
    }
}

#[test]
fn a_complete_response_is_valid() {
    assert!(validate(&board_probes(), &answered(vec![hit("main:slab", 1.0)])).is_ok());
}

#[test]
fn a_wrong_version_is_rejected() {
    let response = Response {
        version: 2,
        ..answered(vec![])
    };
    assert!(validate(&board_probes(), &response).is_err());
}

#[test]
fn a_hit_on_an_unknown_probe_is_rejected() {
    let err = validate(&board_probes(), &answered(vec![hit("main:ghost", 1.0)])).unwrap_err();
    assert!(err.contains("main:ghost"), "{err}");
}

#[test]
fn a_probe_without_a_result_is_rejected() {
    let mut response = answered(vec![]);
    response.measured.pop();
    assert!(validate(&board_probes(), &response).is_err());
}
