use media_resolver_core::{ResolveError, inspect_input, prepare_request, process_response};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PrepareResult {
    key: String,
    normalized_input: String,
    request: media_resolver_core::PreparedRequest,
}

#[derive(Debug, Serialize)]
struct PublicError<'a> {
    code: &'a str,
    message: String,
}

#[wasm_bindgen]
pub fn prepare(input: &str) -> Result<JsValue, JsValue> {
    let descriptor = inspect_input(input).map_err(to_js_error)?;
    let request = prepare_request(&descriptor).map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&PrepareResult {
        key: descriptor.source_key,
        normalized_input: descriptor.normalized_input,
        request,
    })
    .map_err(|_| to_js_error(ResolveError::Internal))
}

#[wasm_bindgen]
pub fn complete(input: &str, status: u16, body: &[u8]) -> Result<JsValue, JsValue> {
    let descriptor = inspect_input(input).map_err(to_js_error)?;
    let result = process_response(&descriptor, status, body).map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&result).map_err(|_| to_js_error(ResolveError::Internal))
}

fn to_js_error(error: ResolveError) -> JsValue {
    serde_wasm_bindgen::to_value(&PublicError {
        code: error.code().as_str(),
        message: error.to_string(),
    })
    .unwrap_or_else(|_| JsValue::from_str("internal"))
}
