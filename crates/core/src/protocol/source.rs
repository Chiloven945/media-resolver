use url::Url;

use crate::{error::ResolveError, model::InputDescriptor};

const CANONICAL_HOST: &str = "x.com";
const INPUT_HOSTS: &[&str] = &[
    "x.com",
    "www.x.com",
    "mobile.x.com",
    "twitter.com",
    "www.twitter.com",
    "mobile.twitter.com",
];
const MAX_SOURCE_KEY_LENGTH: usize = 20;

pub(crate) fn inspect_source(input: &str) -> Result<InputDescriptor, ResolveError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(ResolveError::InvalidInput);
    }

    let url = Url::parse(input).map_err(|_| ResolveError::InvalidInput)?;
    if url.scheme() != "https" && url.scheme() != "http" {
        return Err(ResolveError::UnsupportedInput);
    }

    let host = url.host_str().ok_or(ResolveError::InvalidInput)?;
    if !INPUT_HOSTS.contains(&host) {
        return Err(ResolveError::UnsupportedInput);
    }

    let segments = url
        .path_segments()
        .ok_or(ResolveError::UnsupportedInput)?
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    let source_key = segments
        .windows(2)
        .find_map(|pair| (pair[0] == "status").then_some(pair[1]))
        .ok_or(ResolveError::UnsupportedInput)?;

    validate_source_key(source_key)?;

    Ok(InputDescriptor {
        source_key: source_key.to_owned(),
        normalized_input: format!("https://{CANONICAL_HOST}/i/status/{source_key}"),
    })
}

pub(crate) fn descriptor_from_key(source_key: &str) -> Result<InputDescriptor, ResolveError> {
    validate_source_key(source_key)?;
    Ok(InputDescriptor {
        source_key: source_key.to_owned(),
        normalized_input: format!("https://{CANONICAL_HOST}/i/status/{source_key}"),
    })
}

pub(crate) fn validate_source_key(source_key: &str) -> Result<(), ResolveError> {
    if source_key.len() < 2
        || source_key.len() > MAX_SOURCE_KEY_LENGTH
        || !source_key.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(ResolveError::InvalidInput);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_supported_status_paths() {
        let input = inspect_source("https://x.com/example/status/2011590242474893548/photo/1?x=1")
            .expect("supported input");
        assert_eq!(input.source_key, "2011590242474893548");
    }

    #[test]
    fn rejects_oversized_source_key() {
        let source = "1".repeat(21);
        assert_eq!(
            validate_source_key(&source),
            Err(ResolveError::InvalidInput)
        );
    }
}
