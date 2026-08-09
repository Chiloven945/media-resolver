use crate::{
    error::ResolveError,
    model::{InputDescriptor, PreparedRequest, ResourceBundle},
    resolution::ResolutionContext,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RouteFailure {
    NotFound,
    NoResources,
    RateLimited,
    Unavailable,
    NetworkUnavailable,
    InvalidResponse,
    Rejected,
}

pub(crate) enum AdapterOutcome {
    Resolved(ResourceBundle),
    Fallback(RouteFailure),
    Terminal(ResolveError),
}

pub(crate) trait ResolverAdapter {
    fn prepare(
        &self,
        input: &InputDescriptor,
        context: &ResolutionContext,
        route_key: &str,
    ) -> Result<PreparedRequest, ResolveError>;

    fn process(&self, input: &InputDescriptor, status: u16, body: &[u8]) -> AdapterOutcome;
}
