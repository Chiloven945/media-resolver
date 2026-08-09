use serde::{Deserialize, Serialize};

use crate::{
    error::ResolveError,
    model::{InputDescriptor, PreparedRequest},
    protocol::{
        adapter::{AdapterOutcome, ResolverAdapter},
        routes::{
            embed_v1::EmbedV1Adapter,
            managed_v1::{ManagedV1Adapter, validate_endpoint},
            public_v2::PublicV2Adapter,
        },
    },
    resolution::{ResolutionContext, ResolutionOptions},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum RouteDescriptor {
    PublicV2 { route_key: String },
    ManagedV1 { route_key: String, endpoint: String },
    EmbedV1 { route_key: String },
}

pub(crate) fn build_routes(
    options: &ResolutionOptions,
) -> Result<Vec<RouteDescriptor>, ResolveError> {
    let mut routes = Vec::with_capacity(3);
    routes.push(RouteDescriptor::PublicV2 {
        route_key: route_key(routes.len()),
    });

    if let Some(endpoint) = options
        .gateway_endpoint
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        validate_endpoint(endpoint)?;
        routes.push(RouteDescriptor::ManagedV1 {
            route_key: route_key(routes.len()),
            endpoint: endpoint.to_owned(),
        });
    }

    routes.push(RouteDescriptor::EmbedV1 {
        route_key: route_key(routes.len()),
    });
    Ok(routes)
}

pub(crate) fn prepare(
    route: &RouteDescriptor,
    input: &InputDescriptor,
    context: &ResolutionContext,
) -> Result<PreparedRequest, ResolveError> {
    let _ = context.profile;
    match route {
        RouteDescriptor::PublicV2 { route_key } => {
            PublicV2Adapter.prepare(input, context, route_key)
        }
        RouteDescriptor::ManagedV1 {
            route_key,
            endpoint,
        } => ManagedV1Adapter::new(endpoint).prepare(input, context, route_key),
        RouteDescriptor::EmbedV1 { route_key } => EmbedV1Adapter.prepare(input, context, route_key),
    }
}

pub(crate) fn process(
    route: &RouteDescriptor,
    input: &InputDescriptor,
    status: u16,
    body: &[u8],
) -> AdapterOutcome {
    match route {
        RouteDescriptor::PublicV2 { .. } => PublicV2Adapter.process(input, status, body),
        RouteDescriptor::ManagedV1 { endpoint, .. } => {
            ManagedV1Adapter::new(endpoint).process(input, status, body)
        }
        RouteDescriptor::EmbedV1 { .. } => EmbedV1Adapter.process(input, status, body),
    }
}

fn route_key(index: usize) -> String {
    format!("r{index}")
}
