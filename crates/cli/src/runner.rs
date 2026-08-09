use std::time::Duration;

use futures::{StreamExt, stream};
use media_resolver_core::ResolveError;
use media_resolver_core::ResourceBundle;
use media_resolver_core::inspect_input;
use media_resolver_core::prepare_request;
use media_resolver_core::process_response;
use serde::Serialize;

use crate::client::{RemoteClient, TransportError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultState {
    Ready,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicFailure {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolveResult {
    pub input: String,
    pub state: ResultState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<ResourceBundle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<PublicFailure>,
}

enum TaskFailure {
    Resolve(ResolveError),
    Transport,
}

impl From<ResolveError> for TaskFailure {
    fn from(value: ResolveError) -> Self {
        Self::Resolve(value)
    }
}

impl From<TransportError> for TaskFailure {
    fn from(_value: TransportError) -> Self {
        Self::Transport
    }
}

pub async fn run(
    inputs: Vec<String>,
    jobs: usize,
    timeout: Duration,
    verbose: bool,
) -> Result<Vec<ResolveResult>, reqwest::Error> {
    let client = RemoteClient::new(timeout)?;
    let mut output = stream::iter(inputs.into_iter().enumerate())
        .map(|(index, input)| {
            let client = client.clone();
            async move {
                let result = resolve_one(&client, input, verbose).await;
                (index, result)
            }
        })
        .buffer_unordered(jobs)
        .collect::<Vec<_>>()
        .await;

    output.sort_by_key(|(index, _)| *index);
    Ok(output.into_iter().map(|(_, result)| result).collect())
}

async fn resolve_one(client: &RemoteClient, input: String, verbose: bool) -> ResolveResult {
    if verbose {
        eprintln!("starting task");
    }

    let result: Result<ResourceBundle, TaskFailure> = async {
        let descriptor = inspect_input(&input)?;
        let request = prepare_request(&descriptor)?;
        if verbose {
            eprintln!("request prepared for source {}", descriptor.source_key);
        }

        let (status, body) = client.execute(&request).await?;
        if verbose {
            eprintln!("remote response received with status {status}");
        }

        Ok(process_response(&descriptor, status, &body)?)
    }
    .await;

    match result {
        Ok(bundle) => ResolveResult {
            input,
            state: ResultState::Ready,
            result: Some(bundle),
            error: None,
        },
        Err(error) => ResolveResult {
            input,
            state: ResultState::Failed,
            result: None,
            error: Some(public_failure(error)),
        },
    }
}

fn public_failure(error: TaskFailure) -> PublicFailure {
    match error {
        TaskFailure::Resolve(error) => PublicFailure {
            code: error.code().as_str(),
            message: error.to_string(),
        },
        TaskFailure::Transport => PublicFailure {
            code: "network_error",
            message: "the source could not be reached".to_owned(),
        },
    }
}
