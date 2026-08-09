use std::cmp::Reverse;

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

pub(crate) fn ensure_item_urls(item: &mut ResourceItem) -> Result<(), ResolveError> {
    item.preferred_url = require_https(&item.preferred_url)?;
    item.preview_url = item.preview_url.as_deref().map(require_https).transpose()?;

    for variant in &mut item.variants {
        variant.url = require_https(&variant.url)?;
    }

    Ok(())
}

pub(crate) fn sort_variants(variants: &mut [ResourceVariant]) {
    variants.sort_by_key(|variant| {
        Reverse((
            variant_rank(variant),
            variant
                .width
                .unwrap_or(0)
                .saturating_mul(variant.height.unwrap_or(0)),
            variant.bitrate.unwrap_or(0),
            variant.size_bytes.unwrap_or(0),
        ))
    });
}

pub(crate) fn preferred_variant(variants: &[ResourceVariant]) -> Option<&ResourceVariant> {
    variants.first()
}

pub(crate) fn mime_for_container(container: Option<&str>) -> Option<String> {
    match container?.to_ascii_lowercase().as_str() {
        "mp4" => Some("video/mp4".to_owned()),
        "webm" => Some("video/webm".to_owned()),
        "m3u8" => Some("application/x-mpegURL".to_owned()),
        _ => None,
    }
}

pub(crate) fn container_from_url(input: &str) -> Option<String> {
    let url = Url::parse(input).ok()?;
    let path = url.path().to_ascii_lowercase();
    if path.ends_with(".mp4") {
        Some("mp4".to_owned())
    } else if path.ends_with(".webm") {
        Some("webm".to_owned())
    } else if path.ends_with(".m3u8") {
        Some("m3u8".to_owned())
    } else {
        None
    }
}

pub(crate) fn original_image_url_for_host(
    input: &str,
    rewrite_host: &str,
) -> Result<String, ResolveError> {
    let mut url = Url::parse(input).map_err(|_| ResolveError::InvalidResponse)?;
    if url.scheme() != "https" {
        return Err(ResolveError::InvalidResponse);
    }

    let Some(host) = url.host_str() else {
        return Err(ResolveError::InvalidResponse);
    };

    if host != rewrite_host {
        return Ok(url.to_string());
    }

    let query_format = url
        .query_pairs()
        .find_map(|(name, value)| (name == "format").then(|| value.into_owned()));

    let current_path = url.path().to_owned();
    let (path, format) = if let Some(format) = query_format {
        let normalized = normalize_image_format(&format).ok_or(ResolveError::InvalidResponse)?;
        (current_path, normalized.to_owned())
    } else if let Some((base, extension)) = split_image_extension(&current_path) {
        let normalized = normalize_image_format(extension).ok_or(ResolveError::InvalidResponse)?;
        (base.to_owned(), normalized.to_owned())
    } else {
        return Ok(url.to_string());
    };

    url.set_path(&path);
    url.set_query(None);
    url.query_pairs_mut()
        .append_pair("format", &format)
        .append_pair("name", "orig");
    Ok(url.to_string())
}

pub(crate) fn image_mime_type(input: &str) -> Option<String> {
    let url = Url::parse(input).ok()?;
    let query_format = url
        .query_pairs()
        .find_map(|(name, value)| (name == "format").then(|| value.into_owned()));

    let format = query_format
        .as_deref()
        .and_then(normalize_image_format)
        .or_else(|| {
            let path = url.path().to_ascii_lowercase();
            let (_, extension) = split_image_extension(&path)?;
            normalize_image_format(extension)
        })?;

    match format {
        "jpg" => Some("image/jpeg".to_owned()),
        "png" => Some("image/png".to_owned()),
        "webp" => Some("image/webp".to_owned()),
        _ => None,
    }
}

pub(crate) fn dimensions_from_url(input: &str) -> Option<(Option<u32>, Option<u32>)> {
    let url = Url::parse(input).ok()?;
    url.path_segments()?.find_map(|segment| {
        let (width, height) = segment.split_once('x')?;
        let width = width.parse::<u32>().ok()?;
        let height = height.parse::<u32>().ok()?;
        Some((Some(width), Some(height)))
    })
}

fn variant_rank(variant: &ResourceVariant) -> u8 {
    let container = variant
        .container
        .as_deref()
        .map(str::to_ascii_lowercase)
        .or_else(|| container_from_url(&variant.url));
    let codec = variant.codec.as_deref().map(str::to_ascii_lowercase);
    let mime = variant.mime_type.as_deref().map(str::to_ascii_lowercase);

    match (container.as_deref(), codec.as_deref(), mime.as_deref()) {
        (Some("mp4"), Some("h264"), _) => 6,
        (Some("mp4"), _, _) => 5,
        (_, _, Some("video/mp4")) => 5,
        (Some("webm"), _, _) => 4,
        (_, _, Some("video/webm")) => 4,
        (None, _, Some(value)) if value.starts_with("video/") => 3,
        (Some("m3u8"), _, _) => 1,
        (_, _, Some("application/x-mpegurl" | "application/vnd.apple.mpegurl")) => 1,
        _ => 2,
    }
}

fn normalize_image_format(format: &str) -> Option<&'static str> {
    if format.eq_ignore_ascii_case("jpg") || format.eq_ignore_ascii_case("jpeg") {
        Some("jpg")
    } else if format.eq_ignore_ascii_case("png") {
        Some("png")
    } else if format.eq_ignore_ascii_case("webp") {
        Some("webp")
    } else {
        None
    }
}

fn split_image_extension(path: &str) -> Option<(&str, &str)> {
    let (base, extension) = path.rsplit_once('.')?;
    ["jpg", "jpeg", "png", "webp"]
        .iter()
        .any(|candidate| extension.eq_ignore_ascii_case(candidate))
        .then_some((base, extension))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn variant(
        container: &str,
        codec: Option<&str>,
        bitrate: u64,
        width: u32,
        height: u32,
    ) -> ResourceVariant {
        ResourceVariant {
            url: format!("https://assets.invalid/{width}x{height}/file.{container}"),
            mime_type: mime_for_container(Some(container)),
            container: Some(container.to_owned()),
            codec: codec.map(str::to_owned),
            bitrate: Some(bitrate),
            size_bytes: None,
            width: Some(width),
            height: Some(height),
        }
    }

    #[test]
    fn prefers_direct_h264_mp4_before_other_formats() {
        let mut variants = vec![
            variant("m3u8", None, 8_000_000, 1920, 1080),
            variant("webm", Some("vp9"), 5_000_000, 1920, 1080),
            variant("mp4", Some("h264"), 2_000_000, 1280, 720),
        ];
        sort_variants(&mut variants);
        assert_eq!(variants[0].container.as_deref(), Some("mp4"));
        assert_eq!(variants[0].codec.as_deref(), Some("h264"));
    }

    #[test]
    fn builds_original_image_variant() {
        let output = original_image_url_for_host(
            "https://assets.invalid/media/example.jpg",
            "assets.invalid",
        )
        .unwrap();
        assert_eq!(
            output,
            "https://assets.invalid/media/example?format=jpg&name=orig"
        );
    }
}
