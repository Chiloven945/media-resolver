use crate::{error::ResolveError, model::InputDescriptor, protocol::source};

pub(crate) fn inspect(input: &str) -> Result<InputDescriptor, ResolveError> {
    source::inspect_source(input)
}
