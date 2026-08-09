use serde::{Deserialize, Serialize};

use crate::{
    error::{ResolveError, ResolveErrorCode},
    input,
    model::{InputDescriptor, PreparedRequest, ResourceBundle},
    protocol::{
        adapter::{AdapterOutcome, RouteFailure},
        registry::{self, RouteDescriptor},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeProfile {
    Browser,
    Native,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolutionOptions {
    pub profile: RuntimeProfile,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gateway_endpoint: Option<String>,
}

impl Default for ResolutionOptions {
    fn default() -> Self {
        Self {
            profile: RuntimeProfile::Browser,
            gateway_endpoint: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportFailure {
    Network,
    AccessBlocked,
    Timeout,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolveFailure {
    pub code: ResolveErrorCode,
    pub message: String,
}

impl ResolveFailure {
    fn from_error(error: ResolveError) -> Self {
        Self {
            code: error.code(),
            message: error.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionSession {
    pub schema_version: u32,
    descriptor: InputDescriptor,
    profile: RuntimeProfile,
    route_index: usize,
    routes: Vec<RouteDescriptor>,
    failures: Vec<RouteFailureRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RouteFailureRecord {
    failure: SerializableRouteFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SerializableRouteFailure {
    NotFound,
    NoResources,
    RateLimited,
    Unavailable,
    NetworkUnavailable,
    InvalidResponse,
    Rejected,
}

impl From<RouteFailure> for SerializableRouteFailure {
    fn from(value: RouteFailure) -> Self {
        match value {
            RouteFailure::NotFound => Self::NotFound,
            RouteFailure::NoResources => Self::NoResources,
            RouteFailure::RateLimited => Self::RateLimited,
            RouteFailure::Unavailable => Self::Unavailable,
            RouteFailure::NetworkUnavailable => Self::NetworkUnavailable,
            RouteFailure::InvalidResponse => Self::InvalidResponse,
            RouteFailure::Rejected => Self::Rejected,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ResolutionStep {
    Request {
        session: ResolutionSession,
        request: Box<PreparedRequest>,
        #[serde(rename = "sourceKey")]
        source_key: String,
        #[serde(rename = "normalizedInput")]
        normalized_input: String,
    },
    Resolved {
        result: ResourceBundle,
    },
    Failed {
        error: ResolveFailure,
    },
}

pub(crate) struct ResolutionContext {
    pub(crate) profile: RuntimeProfile,
}

pub fn start_resolution(
    input_value: &str,
    options: ResolutionOptions,
) -> Result<ResolutionStep, ResolveError> {
    let descriptor = input::inspect(input_value)?;
    let routes = registry::build_routes(&options)?;
    let session = ResolutionSession {
        schema_version: 1,
        descriptor,
        profile: options.profile,
        route_index: 0,
        routes,
        failures: Vec::new(),
    };
    request_current(session)
}

pub fn accept_response(
    session: ResolutionSession,
    status: u16,
    body: &[u8],
) -> Result<ResolutionStep, ResolveError> {
    let route = session
        .routes
        .get(session.route_index)
        .ok_or(ResolveError::Internal)?;
    match registry::process(route, &session.descriptor, status, body) {
        AdapterOutcome::Resolved(result) => Ok(ResolutionStep::Resolved { result }),
        AdapterOutcome::Terminal(error) => Ok(failed_step(error)),
        AdapterOutcome::Fallback(failure) => advance(session, failure),
    }
}

pub fn accept_transport_failure(
    session: ResolutionSession,
    failure: TransportFailure,
) -> Result<ResolutionStep, ResolveError> {
    let route_failure = match failure {
        TransportFailure::Network | TransportFailure::AccessBlocked | TransportFailure::Timeout => {
            RouteFailure::NetworkUnavailable
        }
    };
    advance(session, route_failure)
}

fn request_current(session: ResolutionSession) -> Result<ResolutionStep, ResolveError> {
    let route = session
        .routes
        .get(session.route_index)
        .ok_or(ResolveError::Internal)?;
    let context = ResolutionContext {
        profile: session.profile,
    };
    let request = registry::prepare(route, &session.descriptor, &context)?;
    Ok(ResolutionStep::Request {
        source_key: session.descriptor.source_key.clone(),
        normalized_input: session.descriptor.normalized_input.clone(),
        session,
        request: Box::new(request),
    })
}

fn advance(
    mut session: ResolutionSession,
    failure: RouteFailure,
) -> Result<ResolutionStep, ResolveError> {
    session
        .routes
        .get(session.route_index)
        .ok_or(ResolveError::Internal)?;
    session.failures.push(RouteFailureRecord {
        failure: failure.into(),
    });
    session.route_index += 1;

    if session.route_index < session.routes.len() {
        request_current(session)
    } else {
        Ok(failed_step(final_error(&session.failures)))
    }
}

fn final_error(failures: &[RouteFailureRecord]) -> ResolveError {
    let contains = |failure| failures.iter().any(|item| item.failure == failure);

    if contains(SerializableRouteFailure::NotFound) {
        ResolveError::RemoteNotFound
    } else if contains(SerializableRouteFailure::NoResources) {
        ResolveError::NoResources
    } else if contains(SerializableRouteFailure::RateLimited) {
        ResolveError::RateLimited
    } else if contains(SerializableRouteFailure::Rejected) {
        ResolveError::RemoteRejected
    } else if contains(SerializableRouteFailure::Unavailable) {
        ResolveError::RemoteUnavailable
    } else if contains(SerializableRouteFailure::NetworkUnavailable) {
        ResolveError::NetworkUnavailable
    } else if contains(SerializableRouteFailure::InvalidResponse) {
        ResolveError::InvalidResponse
    } else {
        ResolveError::Internal
    }
}

fn failed_step(error: ResolveError) -> ResolutionStep {
    ResolutionStep::Failed {
        error: ResolveFailure::from_error(error),
    }
}
