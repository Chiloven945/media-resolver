use crate::{error::ResolveError, model::InputDescriptor, protocol::remote::RemoteAdapter};

pub(crate) fn inspect(input: &str) -> Result<InputDescriptor, ResolveError> {
    RemoteAdapter::inspect(input)
}
