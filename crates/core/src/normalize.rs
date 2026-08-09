use url::Url;

use crate::{
    error::ResolveError,
    model::{ResourceItem, ResourceVariant},
};

pub(crate) fn require_https(input: &str) -> Result<String, ResolveError> {
    let url = Url::parse(input).map_err(|_| ResolveError::InvalidResponse)?;
    if url.scheme() != "https" {
        return Err(ResolveError::InvalidResponse);
    }

    Ok(url.to_string())
}

pub(crate) fn sort_variants(variants: &mut [ResourceVariant]) {
    variants.sort_by(|left, right| {
        right
            .bitrate
            .unwrap_or(0)
            .cmp(&left.bitrate.unwrap_or(0))
            .then_with(|| right.width.unwrap_or(0).cmp(&left.width.unwrap_or(0)))
            .then_with(|| right.height.unwrap_or(0).cmp(&left.height.unwrap_or(0)))
    });
}

pub(crate) fn ensure_item_urls(item: &mut ResourceItem) -> Result<(), ResolveError> {
    item.preferred_url = require_https(&item.preferred_url)?;
    item.preview_url = item.preview_url.as_deref().map(require_https).transpose()?;

    for variant in &mut item.variants {
        variant.url = require_https(&variant.url)?;
    }

    Ok(())
}
