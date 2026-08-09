use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolveErrorCode {
    InvalidInput,
    UnsupportedInput,
    RemoteNotFound,
    RemoteUnavailable,
    RemoteRestricted,
    RemoteRejected,
    RateLimited,
    InvalidResponse,
    NoResources,
    NetworkUnavailable,
    Internal,
}

impl ResolveErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => "invalid_input",
            Self::UnsupportedInput => "unsupported_input",
            Self::RemoteNotFound => "remote_not_found",
            Self::RemoteUnavailable => "remote_unavailable",
            Self::RemoteRestricted => "remote_restricted",
            Self::RemoteRejected => "remote_rejected",
            Self::RateLimited => "rate_limited",
            Self::InvalidResponse => "invalid_response",
            Self::NoResources => "no_resources",
            Self::NetworkUnavailable => "network_unavailable",
            Self::Internal => "internal",
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ResolveError {
    #[error("the input is not a valid link")]
    InvalidInput,
    #[error("the input is not supported")]
    UnsupportedInput,
    #[error("the source is no longer available")]
    RemoteNotFound,
    #[error("the source is unavailable through the current access methods")]
    RemoteUnavailable,
    #[error("the source requires access that is unavailable in the current session")]
    RemoteRestricted,
    #[error("the remote source rejected the request")]
    RemoteRejected,
    #[error("too many requests")]
    RateLimited,
    #[error("the remote source returned an unexpected response")]
    InvalidResponse,
    #[error("no usable resources were found")]
    NoResources,
    #[error("the remote source could not be reached")]
    NetworkUnavailable,
    #[error("an internal resolver error occurred")]
    Internal,
}

impl ResolveError {
    pub const fn code(&self) -> ResolveErrorCode {
        match self {
            Self::InvalidInput => ResolveErrorCode::InvalidInput,
            Self::UnsupportedInput => ResolveErrorCode::UnsupportedInput,
            Self::RemoteNotFound => ResolveErrorCode::RemoteNotFound,
            Self::RemoteUnavailable => ResolveErrorCode::RemoteUnavailable,
            Self::RemoteRestricted => ResolveErrorCode::RemoteRestricted,
            Self::RemoteRejected => ResolveErrorCode::RemoteRejected,
            Self::RateLimited => ResolveErrorCode::RateLimited,
            Self::InvalidResponse => ResolveErrorCode::InvalidResponse,
            Self::NoResources => ResolveErrorCode::NoResources,
            Self::NetworkUnavailable => ResolveErrorCode::NetworkUnavailable,
            Self::Internal => ResolveErrorCode::Internal,
        }
    }
}
