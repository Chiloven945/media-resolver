use crate::{
    error::ResolveError,
    input,
    model::InputDescriptor,
    resolution::{
        ResolutionOptions, ResolutionSession, ResolutionStep, TransportFailure,
        accept_response as resolution_accept_response,
        accept_transport_failure as resolution_accept_transport_failure,
        start_resolution as resolution_start,
        start_resolution_from_key as resolution_start_from_key,
    },
};

pub fn inspect_input(input: &str) -> Result<InputDescriptor, ResolveError> {
    input::inspect(input)
}

pub fn start_resolution(
    input: &str,
    options: ResolutionOptions,
) -> Result<ResolutionStep, ResolveError> {
    resolution_start(input, options)
}

pub fn start_resolution_from_key(
    source_key: &str,
    options: ResolutionOptions,
) -> Result<ResolutionStep, ResolveError> {
    resolution_start_from_key(source_key, options)
}

pub fn accept_response(
    session: ResolutionSession,
    status: u16,
    body: &[u8],
) -> Result<ResolutionStep, ResolveError> {
    resolution_accept_response(session, status, body)
}

pub fn accept_transport_failure(
    session: ResolutionSession,
    failure: TransportFailure,
) -> Result<ResolutionStep, ResolveError> {
    resolution_accept_transport_failure(session, failure)
}
