use crate::{
    error::ResolveError,
    input,
    model::{InputDescriptor, PreparedRequest, ResourceBundle},
    protocol::remote::RemoteAdapter,
};

pub fn inspect_input(input: &str) -> Result<InputDescriptor, ResolveError> {
    input::inspect(input)
}

pub fn prepare_request(input: &InputDescriptor) -> Result<PreparedRequest, ResolveError> {
    RemoteAdapter::prepare(input)
}

pub fn process_response(
    input: &InputDescriptor,
    status: u16,
    body: &[u8],
) -> Result<ResourceBundle, ResolveError> {
    RemoteAdapter::process(input, status, body)
}
