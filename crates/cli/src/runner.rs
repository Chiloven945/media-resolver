use std::{env, time::Duration};

use futures::{StreamExt, stream};
use media_resolver_core::{
    ResolutionOptions, ResolutionStep, ResolveError, ResolveFailure, ResourceBundle,
    RuntimeProfile, accept_response, accept_transport_failure, start_resolution,
};
use serde::Serialize;

use crate::client::RemoteClient;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultState {
    Ready,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicFailure {
    pub code: String,
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
    Final(ResolveFailure),
}

impl From<ResolveError> for TaskFailure {
    fn from(value: ResolveError) -> Self {
        Self::Resolve(value)
    }
}

pub async fn run(
    inputs: Vec<String>,
    jobs: usize,
    timeout: Duration,
    verbose: bool,
) -> Result<Vec<ResolveResult>, reqwest::Error> {
    let client = RemoteClient::new(timeout)?;
    let gateway_endpoint = env::var("MEDIA_RESOLVER_GATEWAY_ENDPOINT")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    let mut output = stream::iter(inputs.into_iter().enumerate())
        .map(|(index, input)| {
            let client = client.clone();
            let gateway_endpoint = gateway_endpoint.clone();
            async move {
                let result = resolve_one(&client, input, gateway_endpoint, verbose).await;
                (index, result)
            }
        })
        .buffer_unordered(jobs)
        .collect::<Vec<_>>()
        .await;

    output.sort_by_key(|(index, _)| *index);
    Ok(output.into_iter().map(|(_, result)| result).collect())
}

async fn resolve_one(
    client: &RemoteClient,
    input: String,
    gateway_endpoint: Option<String>,
    verbose: bool,
) -> ResolveResult {
    if verbose {
        eprintln!("starting task");
    }

    let result: Result<ResourceBundle, TaskFailure> = async {
        let mut step = start_resolution(
            &input,
            ResolutionOptions {
                profile: RuntimeProfile::Native,
                gateway_endpoint,
            },
        )?;

        loop {
            match step {
                ResolutionStep::Request {
                    session,
                    request,
                    source_key,
                    ..
                } => {
                    if verbose {
                        eprintln!(
                            "request {} prepared for source {}",
                            request.route_key, source_key
                        );
                    }
                    step = match client.execute(&request).await {
                        Ok((status, body)) => {
                            if verbose {
                                eprintln!(
                                    "request {} completed with status {}",
                                    request.route_key, status
                                );
                            }
                            accept_response(session, status, &body)?
                        }
                        Err(error) => {
                            if verbose {
                                eprintln!("request {} transport failed", request.route_key);
                            }
                            accept_transport_failure(session, error.as_core_failure())?
                        }
                    };
                }
                ResolutionStep::Resolved { result } => break Ok(result),
                ResolutionStep::Failed { error } => break Err(TaskFailure::Final(error)),
            }
        }
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
            code: error.code().as_str().to_owned(),
            message: error.to_string(),
        },
        TaskFailure::Final(error) => PublicFailure {
            code: error.code.as_str().to_owned(),
            message: error.message,
        },
    }
}
