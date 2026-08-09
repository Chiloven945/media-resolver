use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolveErrorCode {
    InvalidInput,
    UnsupportedInput,
    RemoteNotFound,
    RemoteRejected,
    RateLimited,
    InvalidResponse,
    NoResources,
    Internal,
}

impl ResolveErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => "invalid_input",
            Self::UnsupportedInput => "unsupported_input",
            Self::RemoteNotFound => "remote_not_found",
            Self::RemoteRejected => "remote_rejected",
            Self::RateLimited => "rate_limited",
            Self::InvalidResponse => "invalid_response",
            Self::NoResources => "no_resources",
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
    #[error("the remote source rejected the request")]
    RemoteRejected,
    #[error("too many requests")]
    RateLimited,
    #[error("the remote source returned an unexpected response")]
    InvalidResponse,
    #[error("no usable resources were found")]
    NoResources,
    #[error("an internal resolver error occurred")]
    Internal,
}

impl ResolveError {
    pub const fn code(&self) -> ResolveErrorCode {
        match self {
            Self::InvalidInput => ResolveErrorCode::InvalidInput,
            Self::UnsupportedInput => ResolveErrorCode::UnsupportedInput,
            Self::RemoteNotFound => ResolveErrorCode::RemoteNotFound,
            Self::RemoteRejected => ResolveErrorCode::RemoteRejected,
            Self::RateLimited => ResolveErrorCode::RateLimited,
            Self::InvalidResponse => ResolveErrorCode::InvalidResponse,
            Self::NoResources => ResolveErrorCode::NoResources,
            Self::Internal => ResolveErrorCode::Internal,
        }
    }
}
