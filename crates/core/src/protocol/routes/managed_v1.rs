use serde_json::Value;
use url::Url;

use crate::{
    error::ResolveError,
    model::{InputDescriptor, PreparedRequest, RequestMethod, ResourceBundle, RetryPolicy},
    normalize::ensure_item_urls,
    protocol::{
        adapter::{AdapterOutcome, ResolverAdapter, RouteFailure},
        schema::managed_v1::ManagedErrorEnvelope,
    },
    resolution::ResolutionContext,
};

pub(crate) struct ManagedV1Adapter<'a> {
    endpoint: &'a str,
}

impl<'a> ManagedV1Adapter<'a> {
    pub(crate) const fn new(endpoint: &'a str) -> Self {
        Self { endpoint }
    }
}

pub(crate) fn validate_endpoint(endpoint: &str) -> Result<(), ResolveError> {
    let url = Url::parse(endpoint).map_err(|_| ResolveError::Internal)?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(ResolveError::Internal);
    }
    Ok(())
}

impl ResolverAdapter for ManagedV1Adapter<'_> {
    fn prepare(
        &self,
        input: &InputDescriptor,
        _context: &ResolutionContext,
        route_key: &str,
    ) -> Result<PreparedRequest, ResolveError> {
        validate_endpoint(self.endpoint)?;
        let url = format!(
            "{}/v1/resources/{}",
            self.endpoint.trim_end_matches('/'),
            input.source_key
        );
        Ok(PreparedRequest {
            route_key: route_key.to_owned(),
            url,
            method: RequestMethod::Get,
            headers: Vec::new(),
            retry_policy: RetryPolicy::default(),
        })
    }

    fn process(&self, input: &InputDescriptor, status: u16, body: &[u8]) -> AdapterOutcome {
        let value: Option<Value> = serde_json::from_slice(body).ok();
        if let Some(value) = value.as_ref()
            && let Ok(envelope) = serde_json::from_value::<ManagedErrorEnvelope>(value.clone())
        {
            return classify_error_code(&envelope.error.code);
        }

        if let Some(outcome) = classify_http_status(status) {
            return outcome;
        }

        let Some(value) = value else {
            return AdapterOutcome::Fallback(RouteFailure::InvalidResponse);
        };
        let mut bundle: ResourceBundle = match serde_json::from_value(value) {
            Ok(value) => value,
            Err(_) => return AdapterOutcome::Fallback(RouteFailure::InvalidResponse),
        };

        if bundle.schema_version != 1 || bundle.source_key != input.source_key {
            return AdapterOutcome::Fallback(RouteFailure::InvalidResponse);
        }
        if bundle.resources.is_empty() {
            return AdapterOutcome::Fallback(RouteFailure::NoResources);
        }
        for item in &mut bundle.resources {
            if ensure_item_urls(item).is_err() {
                return AdapterOutcome::Fallback(RouteFailure::InvalidResponse);
            }
        }
        AdapterOutcome::Resolved(bundle)
    }
}

fn classify_error_code(code: &str) -> AdapterOutcome {
    match code {
        "remote_restricted" => AdapterOutcome::Terminal(ResolveError::RemoteRestricted),
        "remote_not_found" => AdapterOutcome::Fallback(RouteFailure::NotFound),
        "no_resources" => AdapterOutcome::Fallback(RouteFailure::NoResources),
        "rate_limited" => AdapterOutcome::Fallback(RouteFailure::RateLimited),
        "remote_unavailable" => AdapterOutcome::Fallback(RouteFailure::Unavailable),
        "network_unavailable" | "network_error" => {
            AdapterOutcome::Fallback(RouteFailure::NetworkUnavailable)
        }
        "invalid_response" => AdapterOutcome::Fallback(RouteFailure::InvalidResponse),
        "remote_rejected" => AdapterOutcome::Fallback(RouteFailure::Rejected),
        _ => AdapterOutcome::Fallback(RouteFailure::InvalidResponse),
    }
}

fn classify_http_status(status: u16) -> Option<AdapterOutcome> {
    match status {
        200..=299 => None,
        404 => Some(AdapterOutcome::Fallback(RouteFailure::NotFound)),
        429 => Some(AdapterOutcome::Fallback(RouteFailure::RateLimited)),
        400..=499 => Some(AdapterOutcome::Fallback(RouteFailure::Rejected)),
        500..=599 => Some(AdapterOutcome::Fallback(RouteFailure::Unavailable)),
        _ => Some(AdapterOutcome::Fallback(RouteFailure::Rejected)),
    }
}
