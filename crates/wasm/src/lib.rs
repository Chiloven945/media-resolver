use media_resolver_core::{
    ResolutionOptions, ResolutionSession, ResolveError, TransportFailure, accept_response,
    accept_transport_failure, start_resolution,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Debug, Serialize)]
struct PublicError<'a> {
    code: &'a str,
    message: String,
}

#[wasm_bindgen]
pub fn start(input: &str, options: JsValue) -> Result<JsValue, JsValue> {
    let options: ResolutionOptions =
        serde_wasm_bindgen::from_value(options).map_err(|_| to_js_error(ResolveError::Internal))?;
    let step = start_resolution(input, options).map_err(to_js_error)?;
    serialize_step(&step)
}

#[wasm_bindgen]
pub fn respond(session: JsValue, status: u16, body: &[u8]) -> Result<JsValue, JsValue> {
    let session: ResolutionSession =
        serde_wasm_bindgen::from_value(session).map_err(|_| to_js_error(ResolveError::Internal))?;
    let step = accept_response(session, status, body).map_err(to_js_error)?;
    serialize_step(&step)
}

#[wasm_bindgen]
pub fn transport_failed(session: JsValue, kind: &str) -> Result<JsValue, JsValue> {
    let session: ResolutionSession =
        serde_wasm_bindgen::from_value(session).map_err(|_| to_js_error(ResolveError::Internal))?;
    let failure = match kind {
        "network" => TransportFailure::Network,
        "access_blocked" => TransportFailure::AccessBlocked,
        "timeout" => TransportFailure::Timeout,
        _ => return Err(to_js_error(ResolveError::Internal)),
    };
    let step = accept_transport_failure(session, failure).map_err(to_js_error)?;
    serialize_step(&step)
}

fn serialize_step(step: &media_resolver_core::ResolutionStep) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(step).map_err(|_| to_js_error(ResolveError::Internal))
}

fn to_js_error(error: ResolveError) -> JsValue {
    serde_wasm_bindgen::to_value(&PublicError {
        code: error.code().as_str(),
        message: error.to_string(),
    })
    .unwrap_or_else(|_| JsValue::from_str("internal"))
}
