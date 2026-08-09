#![cfg(target_arch = "wasm32")]

use media_resolver_core::{inspect_input, process_response};
use media_resolver_wasm::complete;
use serde_wasm_bindgen::from_value;
use wasm_bindgen_test::*;

const FIXTURE: &[u8] = include_bytes!("../../../tests/fixtures/image-single.json");
const INPUT: &str = include_str!("../../../tests/fixtures/input-canonical.txt");

#[wasm_bindgen_test]
fn wasm_complete_matches_core_serialization() {
    let descriptor = inspect_input(INPUT.trim()).unwrap();
    let expected = process_response(&descriptor, 200, FIXTURE).unwrap();
    let actual: media_resolver_core::ResourceBundle =
        from_value(complete(INPUT.trim(), 200, FIXTURE).unwrap()).unwrap();
    assert_eq!(actual, expected);
}
