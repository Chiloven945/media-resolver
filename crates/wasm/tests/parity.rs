#![cfg(target_arch = "wasm32")]

use media_resolver_core::{
    ResolutionOptions, ResolutionStep, RuntimeProfile, TransportFailure, accept_response,
    accept_transport_failure, start_resolution, start_resolution_from_key,
};
use media_resolver_wasm::{respond, start, start_key, transport_failed};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen_test::*;

const FIXTURE: &[u8] = include_bytes!("../../../tests/fixtures/public-image-single.json");
const INPUT: &str = include_str!("../../../tests/fixtures/input-canonical.txt");

fn options() -> ResolutionOptions {
    ResolutionOptions {
        profile: RuntimeProfile::Browser,
        gateway_endpoint: None,
    }
}

#[wasm_bindgen_test]
fn wasm_start_matches_core() {
    let expected = start_resolution(INPUT.trim(), options()).unwrap();
    let actual: ResolutionStep =
        from_value(start(INPUT.trim(), to_value(&options()).unwrap()).unwrap()).unwrap();
    assert_eq!(actual, expected);
}

#[wasm_bindgen_test]
fn wasm_start_key_matches_core() {
    let source_key = "2011590242474893548";
    let expected = start_resolution_from_key(source_key, options()).unwrap();
    let actual: ResolutionStep =
        from_value(start_key(source_key, to_value(&options()).unwrap()).unwrap()).unwrap();
    assert_eq!(actual, expected);
}

#[wasm_bindgen_test]
fn wasm_respond_matches_core() {
    let initial = start_resolution(INPUT.trim(), options()).unwrap();
    let ResolutionStep::Request { session, .. } = initial else {
        panic!("expected request step");
    };
    let expected = accept_response(session.clone(), 200, FIXTURE).unwrap();
    let actual: ResolutionStep =
        from_value(respond(to_value(&session).unwrap(), 200, FIXTURE).unwrap()).unwrap();
    assert_eq!(actual, expected);
}

#[wasm_bindgen_test]
fn wasm_transport_failure_matches_core() {
    let initial = start_resolution(INPUT.trim(), options()).unwrap();
    let ResolutionStep::Request { session, .. } = initial else {
        panic!("expected request step");
    };
    let expected =
        accept_transport_failure(session.clone(), TransportFailure::AccessBlocked).unwrap();
    let actual: ResolutionStep =
        from_value(transport_failed(to_value(&session).unwrap(), "access_blocked").unwrap())
            .unwrap();
    assert_eq!(actual, expected);
}
