use serde_json::Value;
use url::Url;

use crate::{
    error::ResolveError,
    model::{
        InputDescriptor, PreparedRequest, RequestMethod, ResourceBundle, ResourceItem,
        ResourceKind, ResourceVariant, RetryPolicy,
    },
    normalize::{
        container_from_url, ensure_item_urls, image_mime_type, mime_for_container,
        original_image_url_for_host, preferred_variant, sort_variants,
    },
    protocol::{
        adapter::{AdapterOutcome, ResolverAdapter, RouteFailure},
        schema::public_v2::{
            PublicMedia, PublicPhoto, PublicResponse, PublicStatus, PublicTombstone, PublicVideo,
        },
    },
    resolution::ResolutionContext,
};

const IMAGE_HOST: &str = "pbs.twimg.com";
const ENDPOINT: &str = "https://api.fxtwitter.com/2/status";

pub(crate) struct PublicV2Adapter;

impl ResolverAdapter for PublicV2Adapter {
    fn prepare(
        &self,
        input: &InputDescriptor,
        _context: &ResolutionContext,
        route_key: &str,
    ) -> Result<PreparedRequest, ResolveError> {
        let mut url = Url::parse(ENDPOINT).map_err(|_| ResolveError::Internal)?;
        url.path_segments_mut()
            .map_err(|_| ResolveError::Internal)?
            .push(&input.source_key);

        Ok(PreparedRequest {
            route_key: route_key.to_owned(),
            url: url.to_string(),
            method: RequestMethod::Get,
            headers: Vec::new(),
            retry_policy: RetryPolicy::default(),
        })
    }

    fn process(&self, input: &InputDescriptor, status: u16, body: &[u8]) -> AdapterOutcome {
        if let Some(outcome) = classify_http_status(status) {
            return outcome;
        }

        let response: PublicResponse = match serde_json::from_slice(body) {
            Ok(value) => value,
            Err(_) => return AdapterOutcome::Fallback(RouteFailure::InvalidResponse),
        };

        if let Some(code) = response.code {
            if let Some(outcome) = classify_http_status(code) {
                return outcome;
            }
        }

        let Some(status_value) = response.status else {
            return AdapterOutcome::Fallback(RouteFailure::InvalidResponse);
        };
        process_status_value(input, &status_value, 0)
    }
}

fn process_status_value(
    input: &InputDescriptor,
    status_value: &Value,
    depth: usize,
) -> AdapterOutcome {
    let Some(kind) = status_value.get("type").and_then(Value::as_str) else {
        return AdapterOutcome::Fallback(RouteFailure::InvalidResponse);
    };

    match kind {
        "tombstone" => {
            let tombstone: PublicTombstone = match serde_json::from_value(status_value.clone()) {
                Ok(value) => value,
                Err(_) => return AdapterOutcome::Fallback(RouteFailure::InvalidResponse),
            };
            match tombstone.reason.as_str() {
                "deleted" => AdapterOutcome::Terminal(ResolveError::RemoteNotFound),
                "suspended" => AdapterOutcome::Terminal(ResolveError::RemoteUnavailable),
                "private" | "blocked" => AdapterOutcome::Terminal(ResolveError::RemoteRestricted),
                "unavailable" => AdapterOutcome::Fallback(RouteFailure::Unavailable),
                _ => AdapterOutcome::Fallback(RouteFailure::Unavailable),
            }
        }
        "status" => {
            let document: PublicStatus = match serde_json::from_value(status_value.clone()) {
                Ok(value) => value,
                Err(_) => return AdapterOutcome::Fallback(RouteFailure::InvalidResponse),
            };
            let resources = match document.media.as_ref() {
                Some(media) => match collect_resources(&input.source_key, media) {
                    Ok(items) => items,
                    Err(_) => return AdapterOutcome::Fallback(RouteFailure::InvalidResponse),
                },
                None => Vec::new(),
            };

            if !resources.is_empty() {
                return AdapterOutcome::Resolved(ResourceBundle {
                    schema_version: 1,
                    source_key: input.source_key.clone(),
                    resources,
                });
            }

            if depth < 2 {
                if let Some(quote) = document.quote.as_ref() {
                    if let AdapterOutcome::Resolved(result) =
                        process_status_value(input, quote, depth + 1)
                    {
                        return AdapterOutcome::Resolved(result);
                    }
                }
            }

            AdapterOutcome::Fallback(RouteFailure::NoResources)
        }
        _ => AdapterOutcome::Fallback(RouteFailure::InvalidResponse),
    }
}

fn collect_resources(
    source_key: &str,
    media: &PublicMedia,
) -> Result<Vec<ResourceItem>, ResolveError> {
    let mut resources = Vec::new();

    if !media.all.is_empty() {
        for item in &media.all {
            let Some(kind) = item.get("type").and_then(Value::as_str) else {
                continue;
            };
            let resource = match kind {
                "photo" => serde_json::from_value::<PublicPhoto>(item.clone())
                    .ok()
                    .map(|photo| normalize_photo(source_key, resources.len(), &photo))
                    .transpose()?,
                "video" => serde_json::from_value::<PublicVideo>(item.clone())
                    .ok()
                    .map(|video| normalize_video(source_key, resources.len(), &video))
                    .transpose()?,
                "gif" if item.get("formats").is_some() => {
                    serde_json::from_value::<PublicVideo>(item.clone())
                        .ok()
                        .map(|video| normalize_video(source_key, resources.len(), &video))
                        .transpose()?
                }
                "gif" => serde_json::from_value::<PublicPhoto>(item.clone())
                    .ok()
                    .map(|photo| normalize_photo(source_key, resources.len(), &photo))
                    .transpose()?,
                _ => None,
            };
            if let Some(resource) = resource {
                resources.push(resource.expect("REASON"));
            }
        }
    }

    if resources.is_empty() {
        for photo in &media.photos {
            if let Some(resource) = normalize_photo(source_key, resources.len(), photo)? {
                resources.push(resource);
            }
        }
        for video in &media.videos {
            if let Some(resource) = normalize_video(source_key, resources.len(), video)? {
                resources.push(resource);
            }
        }
    }

    Ok(resources)
}

fn normalize_photo(
    source_key: &str,
    index: usize,
    photo: &PublicPhoto,
) -> Result<Option<ResourceItem>, ResolveError> {
    let id = photo
        .id
        .clone()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| format!("{source_key}:{}", index + 1));

    if photo.kind == "gif" {
        if let Some(transcode_url) = photo.transcode_url.as_deref() {
            let container = container_from_url(transcode_url);
            let mut variants = vec![ResourceVariant {
                url: transcode_url.to_owned(),
                mime_type: mime_for_container(container.as_deref()),
                container,
                codec: None,
                bitrate: None,
                size_bytes: None,
                width: Some(photo.width),
                height: Some(photo.height),
            }];
            sort_variants(&mut variants);
            let Some(preferred) = preferred_variant(&variants) else {
                return Ok(None);
            };
            let mut item = ResourceItem {
                id,
                kind: ResourceKind::Animation,
                preferred_url: preferred.url.clone(),
                preview_url: Some(photo.url.clone()),
                width: Some(photo.width),
                height: Some(photo.height),
                duration_ms: None,
                variants,
            };
            ensure_item_urls(&mut item)?;
            return Ok(Some(item));
        }
    }

    let preferred_url = original_image_url_for_host(&photo.url, IMAGE_HOST)?;
    let mut item = ResourceItem {
        id,
        kind: ResourceKind::Image,
        preferred_url: preferred_url.clone(),
        preview_url: Some(photo.url.clone()),
        width: Some(photo.width),
        height: Some(photo.height),
        duration_ms: None,
        variants: vec![ResourceVariant {
            url: preferred_url,
            mime_type: image_mime_type(&photo.url),
            container: None,
            codec: None,
            bitrate: None,
            size_bytes: None,
            width: Some(photo.width),
            height: Some(photo.height),
        }],
    };
    ensure_item_urls(&mut item)?;
    Ok(Some(item))
}

fn normalize_video(
    source_key: &str,
    index: usize,
    video: &PublicVideo,
) -> Result<Option<ResourceItem>, ResolveError> {
    let mut variants = video
        .formats
        .iter()
        .map(|format| {
            let container = format
                .container
                .as_deref()
                .map(str::to_ascii_lowercase)
                .or_else(|| container_from_url(&format.url));
            ResourceVariant {
                url: format.url.clone(),
                mime_type: mime_for_container(container.as_deref()),
                container,
                codec: format.codec.as_deref().map(str::to_ascii_lowercase),
                bitrate: format.bitrate,
                size_bytes: format.size,
                width: format.width,
                height: format.height,
            }
        })
        .collect::<Vec<_>>();

    if !variants.iter().any(|variant| variant.url == video.url) {
        let container = container_from_url(&video.url);
        variants.push(ResourceVariant {
            url: video.url.clone(),
            mime_type: mime_for_container(container.as_deref()),
            container,
            codec: None,
            bitrate: None,
            size_bytes: video.filesize,
            width: Some(video.width),
            height: Some(video.height),
        });
    }

    sort_variants(&mut variants);
    let Some(preferred) = preferred_variant(&variants) else {
        return Ok(None);
    };

    let duration_ms = video.duration.and_then(duration_ms_from_seconds);
    let mut item = ResourceItem {
        id: video
            .id
            .clone()
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| format!("{source_key}:{}", index + 1)),
        kind: if video.kind == "gif" {
            ResourceKind::Animation
        } else {
            ResourceKind::Video
        },
        preferred_url: preferred.url.clone(),
        preview_url: video.thumbnail_url.clone(),
        width: Some(video.width),
        height: Some(video.height),
        duration_ms,
        variants,
    };
    ensure_item_urls(&mut item)?;
    Ok(Some(item))
}

fn duration_ms_from_seconds(seconds: f64) -> Option<u64> {
    if !seconds.is_finite() || seconds < 0.0 {
        return None;
    }
    Some((seconds * 1_000.0).round() as u64)
}

fn classify_http_status(status: u16) -> Option<AdapterOutcome> {
    match status {
        200..=299 => None,
        404 => Some(AdapterOutcome::Fallback(RouteFailure::NotFound)),
        429 => Some(AdapterOutcome::Fallback(RouteFailure::RateLimited)),
        400..=499 => Some(AdapterOutcome::Fallback(RouteFailure::Rejected)),
        500..=599 => Some(AdapterOutcome::Fallback(RouteFailure::Unavailable)),
        _ => Some(AdapterOutcome::Fallback(RouteFailure::Rejected)),
    }
}
