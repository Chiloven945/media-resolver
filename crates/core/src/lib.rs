mod api;
mod error;
mod input;
mod model;
mod normalize;
mod protocol;

pub use api::{inspect_input, prepare_request, process_response};
pub use error::{ResolveError, ResolveErrorCode};
pub use model::{
    InputDescriptor, PreparedRequest, RequestHeader, RequestMethod, ResourceBundle, ResourceItem,
    ResourceKind, ResourceVariant,
};
