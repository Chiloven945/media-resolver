use std::f64::consts::PI;

use serde_json::Value;
use url::Url;

use crate::{
    error::ResolveError,
    model::{
        InputDescriptor, PreparedRequest, RequestMethod, ResourceBundle, ResourceItem,
        ResourceKind, ResourceVariant, RetryPolicy,
    },
    normalize::{
        container_from_url, dimensions_from_url, ensure_item_urls, image_mime_type,
        mime_for_container, original_image_url_for_host, preferred_variant, sort_variants,
    },
    protocol::{
        adapter::{AdapterOutcome, ResolverAdapter, RouteFailure},
        schema::embed_v1::{EmbedDocument, EmbedMedia, EmbedVariant},
        source::validate_source_key,
    },
    resolution::ResolutionContext,
};

const IMAGE_HOST: &str = "pbs.twimg.com";
const ENDPOINT: &str = "https://cdn.syndication.twimg.com/tweet-result";
const TOMBSTONE_TYPE: &str = "TweetTombstone";
const LANGUAGE: &str = "en";
const FEATURES: &[&str] = &[
    "tfw_timeline_list:",
    "tfw_follower_count_sunset:true",
    "tfw_tweet_edit_backend:on",
    "tfw_refsrc_session:on",
    "tfw_fosnr_soft_interventions_enabled:on",
    "tfw_show_birdwatch_pivots_enabled:on",
    "tfw_show_business_verified_badge:on",
    "tfw_duplicate_scribes_to_settings:on",
    "tfw_use_profile_image_shape_enabled:on",
    "tfw_show_blue_verified_badge:on",
    "tfw_legacy_timeline_sunset:true",
    "tfw_show_gov_verified_badge:on",
    "tfw_show_business_affiliate_badge:on",
    "tfw_tweet_edit_frontend:on",
];

pub(crate) struct EmbedV1Adapter;

impl ResolverAdapter for EmbedV1Adapter {
    fn prepare(
        &self,
        input: &InputDescriptor,
        _context: &ResolutionContext,
        route_key: &str,
    ) -> Result<PreparedRequest, ResolveError> {
        validate_source_key(&input.source_key)?;
        let mut url = Url::parse(ENDPOINT).map_err(|_| ResolveError::Internal)?;
        let features = FEATURES.join(";");
        let token = generate_token(&input.source_key)?;
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("id", &input.source_key);
            query.append_pair("lang", LANGUAGE);
            query.append_pair("features", &features);
            query.append_pair("token", &token);
        }

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

        let value: Value = match serde_json::from_slice(body) {
            Ok(value) => value,
            Err(_) => return AdapterOutcome::Fallback(RouteFailure::InvalidResponse),
        };
        if value.as_object().is_some_and(|object| object.is_empty()) {
            return AdapterOutcome::Fallback(RouteFailure::NotFound);
        }

        let document: EmbedDocument = match serde_json::from_value(value) {
            Ok(value) => value,
            Err(_) => return AdapterOutcome::Fallback(RouteFailure::InvalidResponse),
        };

        if document.type_name.as_deref() == Some(TOMBSTONE_TYPE) {
            return AdapterOutcome::Fallback(RouteFailure::Unavailable);
        }

        let mut resources = match collect_resources(&input.source_key, &document.media_details) {
            Ok(value) => value,
            Err(_) => return AdapterOutcome::Fallback(RouteFailure::InvalidResponse),
        };
        if resources.is_empty() {
            if let Some(quoted) = document.quoted_tweet.as_deref() {
                resources = match collect_resources(&input.source_key, &quoted.media_details) {
                    Ok(value) => value,
                    Err(_) => return AdapterOutcome::Fallback(RouteFailure::InvalidResponse),
                };
            }
        }

        if resources.is_empty() {
            return AdapterOutcome::Fallback(RouteFailure::NoResources);
        }

        AdapterOutcome::Resolved(ResourceBundle {
            schema_version: 1,
            source_key: input.source_key.clone(),
            resources,
        })
    }
}

fn collect_resources(
    source_key: &str,
    media: &[EmbedMedia],
) -> Result<Vec<ResourceItem>, ResolveError> {
    let mut resources = Vec::with_capacity(media.len());
    for (index, item) in media.iter().enumerate() {
        if let Some(mut normalized) = normalize_media(source_key, index, item)? {
            ensure_item_urls(&mut normalized)?;
            resources.push(normalized);
        }
    }
    Ok(resources)
}

fn normalize_media(
    source_key: &str,
    index: usize,
    media: &EmbedMedia,
) -> Result<Option<ResourceItem>, ResolveError> {
    let dimensions = media
        .original_info
        .as_ref()
        .map(|info| (info.width, info.height))
        .unwrap_or((None, None));

    match media.kind.as_str() {
        "photo" => {
            let Some(source_url) = media.media_url_https.as_deref() else {
                return Ok(None);
            };
            let preferred_url = original_image_url_for_host(source_url, IMAGE_HOST)?;
            Ok(Some(ResourceItem {
                id: format!("{source_key}:{}", index + 1),
                kind: ResourceKind::Image,
                preferred_url: preferred_url.clone(),
                preview_url: Some(source_url.to_owned()),
                width: dimensions.0,
                height: dimensions.1,
                duration_ms: None,
                variants: vec![ResourceVariant {
                    url: preferred_url,
                    mime_type: image_mime_type(source_url),
                    container: None,
                    codec: None,
                    bitrate: None,
                    size_bytes: None,
                    width: dimensions.0,
                    height: dimensions.1,
                }],
            }))
        }
        "video" | "animated_gif" => {
            let Some(video_info) = media.video_info.as_ref() else {
                return Ok(None);
            };
            let mut variants = video_info
                .variants
                .iter()
                .map(normalize_variant)
                .collect::<Vec<_>>();
            sort_variants(&mut variants);
            let Some(preferred) = preferred_variant(&variants) else {
                return Ok(None);
            };
            Ok(Some(ResourceItem {
                id: format!("{source_key}:{}", index + 1),
                kind: if media.kind == "animated_gif" {
                    ResourceKind::Animation
                } else {
                    ResourceKind::Video
                },
                preferred_url: preferred.url.clone(),
                preview_url: media.media_url_https.clone(),
                width: dimensions.0.or(preferred.width),
                height: dimensions.1.or(preferred.height),
                duration_ms: video_info.duration_millis,
                variants,
            }))
        }
        _ => Ok(None),
    }
}

fn normalize_variant(variant: &EmbedVariant) -> ResourceVariant {
    let container = container_from_url(&variant.url).or_else(|| {
        variant
            .content_type
            .as_deref()
            .and_then(container_from_mime_type)
            .map(str::to_owned)
    });
    let mime_type = variant
        .content_type
        .clone()
        .or_else(|| mime_for_container(container.as_deref()));
    let (width, height) = dimensions_from_url(&variant.url).unwrap_or((None, None));
    ResourceVariant {
        url: variant.url.clone(),
        mime_type,
        container,
        codec: None,
        bitrate: variant.bitrate,
        size_bytes: None,
        width,
        height,
    }
}

fn container_from_mime_type(value: &str) -> Option<&'static str> {
    if value.eq_ignore_ascii_case("video/mp4") {
        Some("mp4")
    } else if value.eq_ignore_ascii_case("video/webm") {
        Some("webm")
    } else if value.eq_ignore_ascii_case("application/x-mpegURL")
        || value.eq_ignore_ascii_case("application/vnd.apple.mpegurl")
    {
        Some("m3u8")
    } else {
        None
    }
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

fn generate_token(source_key: &str) -> Result<String, ResolveError> {
    validate_source_key(source_key)?;
    let value = source_key
        .parse::<f64>()
        .map_err(|_| ResolveError::InvalidInput)?
        / 1e15
        * PI;
    Ok(float_to_base36(value).replace('0', "").replace('.', ""))
}

fn float_to_base36(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_owned();
    }
    if value == 0.0 {
        return "0".to_owned();
    }
    if value.is_infinite() {
        return if value.is_sign_negative() {
            "-Infinity".to_owned()
        } else {
            "Infinity".to_owned()
        };
    }

    const RADIX: i32 = 36;
    const DIGITS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let sign = value.is_sign_negative();
    let value = value.abs();
    let mut integer = value.trunc();
    let mut fraction = value.fract();
    let ulp = f64::from_bits(value.to_bits() + 1) - value;
    let mut delta = f64::from_bits(1).max(ulp / 2.0);
    let mut fraction_digits = Vec::<i32>::new();

    while fraction >= delta {
        delta *= f64::from(RADIX);
        let expanded = fraction * f64::from(RADIX);
        let digit = expanded.trunc() as i32;
        fraction = expanded.fract();
        fraction_digits.push(digit);
        let needs_rounding = fraction > 0.5 || (fraction == 0.5 && digit & 1 == 1);
        if needs_rounding && fraction + delta > 1.0 {
            let mut carried = false;
            for index in (0..fraction_digits.len()).rev() {
                if fraction_digits[index] + 1 < RADIX {
                    fraction_digits[index] += 1;
                    carried = true;
                    break;
                }
                fraction_digits.pop();
            }
            if !carried {
                integer += 1.0;
            }
            break;
        }
    }

    let mut integer_value = integer as u64;
    let mut integer_digits = Vec::<u8>::new();
    loop {
        integer_digits.push((integer_value % RADIX as u64) as u8);
        integer_value /= RADIX as u64;
        if integer_value == 0 {
            break;
        }
    }
    integer_digits.reverse();

    let mut output = String::new();
    if sign {
        output.push('-');
    }
    output.extend(
        integer_digits
            .into_iter()
            .map(|digit| DIGITS[digit as usize] as char),
    );
    if !fraction_digits.is_empty() {
        output.push('.');
        output.extend(
            fraction_digits
                .into_iter()
                .map(|digit| DIGITS[digit as usize] as char),
        );
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_token_matches_public_client_number_semantics() {
        let cases = [
            ("2011590242474893548", "4vjlhvle61f"),
            ("1234567890123456789", "2zqic77uqyk"),
            ("999999999999999999", "2f9lc2ug9mm"),
        ];
        for (source_key, expected) in cases {
            assert_eq!(generate_token(source_key).unwrap(), expected);
        }
    }
}
