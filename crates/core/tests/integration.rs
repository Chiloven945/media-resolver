use media_resolver_core::{
    ResolveError, ResourceKind, inspect_input, prepare_request, process_response,
};

const IMAGE_SINGLE: &[u8] = include_bytes!("../../../tests/fixtures/image-single.json");
const IMAGE_MULTIPLE: &[u8] = include_bytes!("../../../tests/fixtures/image-multiple.json");
const VIDEO_BASIC: &[u8] = include_bytes!("../../../tests/fixtures/video-basic.json");
const ANIMATION_BASIC: &[u8] = include_bytes!("../../../tests/fixtures/animation-basic.json");
const MISSING: &[u8] = include_bytes!("../../../tests/fixtures/missing.json");
const MALFORMED: &[u8] = include_bytes!("../../../tests/fixtures/malformed.json");
const UNEXPECTED: &[u8] = include_bytes!("../../../tests/fixtures/unexpected.json");
const INPUT_CANONICAL: &str = include_str!("../../../tests/fixtures/input-canonical.txt");
const INPUT_ALTERNATE: &str = include_str!("../../../tests/fixtures/input-alternate.txt");
const INPUT_UNSUPPORTED: &str = include_str!("../../../tests/fixtures/input-unsupported.txt");
const INPUT_INVALID_KEY: &str = include_str!("../../../tests/fixtures/input-invalid-key.txt");
const INPUT_MISSING_KEY: &str = include_str!("../../../tests/fixtures/input-missing-key.txt");

fn input() -> media_resolver_core::InputDescriptor {
    inspect_input(INPUT_CANONICAL.trim()).unwrap()
}

#[test]
fn accepts_alternate_host_query_and_suffix() {
    let parsed = inspect_input(INPUT_ALTERNATE.trim()).unwrap();
    assert_eq!(parsed.source_key, "2011590242474893548");
    assert!(
        parsed
            .normalized_input
            .ends_with("/i/status/2011590242474893548")
    );
}

#[test]
fn rejects_invalid_and_unsupported_inputs() {
    assert_eq!(inspect_input("not a url"), Err(ResolveError::InvalidInput));
    assert_eq!(
        inspect_input(INPUT_UNSUPPORTED.trim()),
        Err(ResolveError::UnsupportedInput)
    );
    assert_eq!(
        inspect_input(INPUT_INVALID_KEY.trim()),
        Err(ResolveError::InvalidInput)
    );
    assert_eq!(
        inspect_input(INPUT_MISSING_KEY.trim()),
        Err(ResolveError::UnsupportedInput)
    );
}

#[test]
fn prepares_generic_request() {
    let request = prepare_request(&input()).unwrap();
    assert_eq!(request.key, "2011590242474893548");
    assert!(request.url.starts_with("https://"));
    assert!(request.url.contains("id=2011590242474893548"));
}

#[test]
fn normalizes_original_image() {
    let bundle = process_response(&input(), 200, IMAGE_SINGLE).unwrap();
    assert_eq!(bundle.schema_version, 1);
    assert_eq!(bundle.resources.len(), 1);
    let resource = &bundle.resources[0];
    assert_eq!(resource.kind, ResourceKind::Image);
    assert_eq!(resource.width, Some(2048));
    assert_eq!(resource.height, Some(2048));
    assert!(resource.preferred_url.contains("name=orig"));
}

#[test]
fn supports_multiple_images() {
    let bundle = process_response(&input(), 200, IMAGE_MULTIPLE).unwrap();
    assert_eq!(bundle.resources.len(), 2);
}

#[test]
fn selects_best_video_and_preserves_variants() {
    let bundle = process_response(&input(), 200, VIDEO_BASIC).unwrap();
    let resource = &bundle.resources[0];
    assert_eq!(resource.kind, ResourceKind::Video);
    assert_eq!(resource.duration_ms, Some(12345));
    assert_eq!(resource.variants.len(), 3);
    assert_eq!(resource.variants[0].bitrate, Some(2176000));
    assert_eq!(resource.variants[0].width, Some(1280));
    assert_eq!(resource.variants[0].height, Some(720));
    assert_eq!(resource.preferred_url, resource.variants[0].url);
}

#[test]
fn represents_animation_with_mp4_variant() {
    let bundle = process_response(&input(), 200, ANIMATION_BASIC).unwrap();
    assert_eq!(bundle.resources[0].kind, ResourceKind::Animation);
    assert_eq!(
        bundle.resources[0].variants[0].mime_type.as_deref(),
        Some("video/mp4")
    );
}

#[test]
fn maps_remote_and_response_errors() {
    assert_eq!(
        process_response(&input(), 404, b"{}"),
        Err(ResolveError::RemoteNotFound)
    );
    assert_eq!(
        process_response(&input(), 429, b"{}"),
        Err(ResolveError::RateLimited)
    );
    assert_eq!(
        process_response(&input(), 403, b"{}"),
        Err(ResolveError::RemoteRejected)
    );
    assert_eq!(
        process_response(&input(), 200, MISSING),
        Err(ResolveError::RemoteNotFound)
    );
    assert_eq!(
        process_response(&input(), 200, MALFORMED),
        Err(ResolveError::InvalidResponse)
    );
    assert_eq!(
        process_response(&input(), 200, UNEXPECTED),
        Err(ResolveError::InvalidResponse)
    );
}

#[test]
fn serialized_schema_uses_camel_case() {
    let bundle = process_response(&input(), 200, IMAGE_SINGLE).unwrap();
    let value = serde_json::to_value(bundle).unwrap();
    assert!(value.get("schemaVersion").is_some());
    assert!(value.get("sourceKey").is_some());
    assert!(value["resources"][0].get("preferredUrl").is_some());
}
