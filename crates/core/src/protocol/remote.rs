use std::f64::consts::PI;

use serde_json::Value;
use url::Url;

use crate::{
    error::ResolveError,
    model::{
        InputDescriptor, PreparedRequest, RequestMethod, ResourceBundle, ResourceItem,
        ResourceKind, ResourceVariant,
    },
    normalize::{ensure_item_urls, sort_variants},
    protocol::{
        request::{FEATURES, LANGUAGE},
        response::{RemoteDocument, RemoteMedia, RemoteVariant},
    },
};

const ENDPOINT: &str = "https://cdn.syndication.twimg.com/tweet-result";
const CANONICAL_HOST: &str = "x.com";
const INPUT_HOSTS: &[&str] = &[
    "x.com",
    "www.x.com",
    "mobile.x.com",
    "twitter.com",
    "www.twitter.com",
    "mobile.twitter.com",
];
const TOMBSTONE_TYPE: &str = "TweetTombstone";
const MAX_SOURCE_KEY_LENGTH: usize = 19;

pub(crate) struct RemoteAdapter;

impl RemoteAdapter {
    pub(crate) fn inspect(input: &str) -> Result<InputDescriptor, ResolveError> {
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

    pub(crate) fn prepare(input: &InputDescriptor) -> Result<PreparedRequest, ResolveError> {
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
            key: input.source_key.clone(),
            url: url.to_string(),
            method: RequestMethod::Get,
            headers: Vec::new(),
        })
    }

    pub(crate) fn process(
        input: &InputDescriptor,
        status: u16,
        body: &[u8],
    ) -> Result<ResourceBundle, ResolveError> {
        match status {
            404 => return Err(ResolveError::RemoteNotFound),
            429 => return Err(ResolveError::RateLimited),
            400..=499 => return Err(ResolveError::RemoteRejected),
            500..=599 => return Err(ResolveError::RemoteRejected),
            200..=299 => {}
            _ => return Err(ResolveError::RemoteRejected),
        }

        let value: Value =
            serde_json::from_slice(body).map_err(|_| ResolveError::InvalidResponse)?;
        if value.as_object().is_some_and(|object| object.is_empty()) {
            return Err(ResolveError::RemoteNotFound);
        }

        let document: RemoteDocument =
            serde_json::from_value(value).map_err(|_| ResolveError::InvalidResponse)?;

        if document.type_name.as_deref() == Some(TOMBSTONE_TYPE) {
            return Err(ResolveError::RemoteNotFound);
        }

        let mut resources = collect_resources(&input.source_key, &document.media_details)?;
        if resources.is_empty()
            && let Some(quoted) = document.quoted_tweet.as_deref()
        {
            resources = collect_resources(&input.source_key, &quoted.media_details)?;
        }

        if resources.is_empty() {
            return Err(ResolveError::NoResources);
        }

        Ok(ResourceBundle {
            schema_version: 1,
            source_key: input.source_key.clone(),
            resources,
        })
    }
}

fn validate_source_key(source_key: &str) -> Result<(), ResolveError> {
    if source_key.is_empty()
        || source_key.len() > MAX_SOURCE_KEY_LENGTH
        || !source_key.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(ResolveError::InvalidInput);
    }
    Ok(())
}

fn collect_resources(
    source_key: &str,
    media: &[RemoteMedia],
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
    media: &RemoteMedia,
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

            let preferred_url = original_image_url(source_url)?;
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
                    bitrate: None,
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
                .filter_map(normalize_variant)
                .collect::<Vec<_>>();
            sort_variants(&mut variants);

            let preferred = variants
                .iter()
                .find(|variant| variant.mime_type.as_deref() == Some("video/mp4"))
                .or_else(|| variants.first())
                .ok_or(ResolveError::InvalidResponse)?;

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

fn normalize_variant(variant: &RemoteVariant) -> Option<ResourceVariant> {
    let mime_type = variant.content_type.clone();
    let is_direct = mime_type
        .as_deref()
        .is_some_and(|value| value.eq_ignore_ascii_case("video/mp4"));
    if !is_direct {
        return None;
    }

    let (width, height) = dimensions_from_url(&variant.url).unwrap_or((None, None));
    Some(ResourceVariant {
        url: variant.url.clone(),
        mime_type,
        bitrate: variant.bitrate,
        width,
        height,
    })
}

fn original_image_url(input: &str) -> Result<String, ResolveError> {
    let mut url = Url::parse(input).map_err(|_| ResolveError::InvalidResponse)?;
    if url.scheme() != "https" {
        return Err(ResolveError::InvalidResponse);
    }

    let Some(host) = url.host_str() else {
        return Err(ResolveError::InvalidResponse);
    };

    if host != "pbs.twimg.com" {
        return Ok(url.to_string());
    }

    let query_format = url.query_pairs().find_map(|(name, value)| {
        if name == "format" {
            Some(value.into_owned())
        } else {
            None
        }
    });

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

fn image_mime_type(input: &str) -> Option<String> {
    let url = Url::parse(input).ok()?;
    let query_format = url.query_pairs().find_map(|(name, value)| {
        if name == "format" {
            Some(value.into_owned())
        } else {
            None
        }
    });

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

fn dimensions_from_url(input: &str) -> Option<(Option<u32>, Option<u32>)> {
    let url = Url::parse(input).ok()?;
    url.path_segments()?.find_map(|segment| {
        let (width, height) = segment.split_once('x')?;
        let width = width.parse::<u32>().ok()?;
        let height = height.parse::<u32>().ok()?;
        Some((Some(width), Some(height)))
    })
}

fn generate_token(source_key: &str) -> Result<String, ResolveError> {
    validate_source_key(source_key)?;

    // This mirrors the public embed client's numeric token formula while keeping
    // the large identifier as text everywhere else in the application.
    let value = source_key
        .parse::<f64>()
        .map_err(|_| ResolveError::InvalidInput)?
        / 1e15
        * PI;
    Ok(float_to_base36(value).replace(['0', '.'], ""))
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
    fn identifies_supported_status_paths() {
        let input =
            RemoteAdapter::inspect("https://x.com/example/status/2011590242474893548/photo/1?x=1")
                .expect("supported input");
        assert_eq!(input.source_key, "2011590242474893548");
    }

    #[test]
    fn rejects_oversized_source_key() {
        let source = "1".repeat(20);
        assert_eq!(
            validate_source_key(&source),
            Err(ResolveError::InvalidInput)
        );
    }

    #[test]
    fn builds_original_image_variant() {
        let output = original_image_url("https://pbs.twimg.com/media/example.jpg").unwrap();
        assert_eq!(
            output,
            "https://pbs.twimg.com/media/example?format=jpg&name=orig"
        );
    }

    #[test]
    fn upgrades_query_style_image_to_original_size() {
        let output =
            original_image_url("https://pbs.twimg.com/media/example?format=png&name=small")
                .unwrap();
        assert_eq!(
            output,
            "https://pbs.twimg.com/media/example?format=png&name=orig"
        );
    }

    #[test]
    fn detects_query_style_image_mime_type() {
        assert_eq!(
            image_mime_type("https://pbs.twimg.com/media/example?format=png&name=small").as_deref(),
            Some("image/png")
        );
    }

    #[test]
    fn parses_dimensions_from_variant_url() {
        assert_eq!(
            dimensions_from_url("https://video.twimg.com/path/vid/avc1/1280x720/file.mp4"),
            Some((Some(1280), Some(720)))
        );
    }

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
