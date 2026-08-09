use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ManagedErrorEnvelope {
    pub error: ManagedError,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ManagedError {
    pub code: String,
}
