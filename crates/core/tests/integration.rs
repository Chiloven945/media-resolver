use media_resolver_core::{
    accept_response, accept_transport_failure, inspect_input, start_resolution, start_resolution_from_key,
    ResolutionOptions, ResolutionStep, ResolveError, ResolveErrorCode, ResourceKind,
    RuntimeProfile, TransportFailure,
};

const INPUT_CANONICAL: &str = include_str!("../../../tests/fixtures/input-canonical.txt");
const INPUT_ALTERNATE: &str = include_str!("../../../tests/fixtures/input-alternate.txt");
const INPUT_UNSUPPORTED: &str = include_str!("../../../tests/fixtures/input-unsupported.txt");
const INPUT_INVALID_KEY: &str = include_str!("../../../tests/fixtures/input-invalid-key.txt");
const INPUT_MISSING_KEY: &str = include_str!("../../../tests/fixtures/input-missing-key.txt");
const PUBLIC_IMAGE: &[u8] = include_bytes!("../../../tests/fixtures/public-image-single.json");
const PUBLIC_IMAGES: &[u8] = include_bytes!("../../../tests/fixtures/public-image-multiple.json");
const PUBLIC_VIDEO: &[u8] = include_bytes!("../../../tests/fixtures/public-video.json");
const PUBLIC_ANIMATION: &[u8] = include_bytes!("../../../tests/fixtures/public-animation.json");
const PUBLIC_ANIMATION_TRANSCODE: &[u8] =
    include_bytes!("../../../tests/fixtures/public-animation-transcode.json");
const PUBLIC_NO_MEDIA: &[u8] = include_bytes!("../../../tests/fixtures/public-no-media.json");
const PUBLIC_UNAVAILABLE: &[u8] =
    include_bytes!("../../../tests/fixtures/public-tombstone-unavailable.json");
const PUBLIC_PRIVATE: &[u8] =
    include_bytes!("../../../tests/fixtures/public-tombstone-private.json");
const PUBLIC_DELETED: &[u8] =
    include_bytes!("../../../tests/fixtures/public-tombstone-deleted.json");
const PUBLIC_BLOCKED: &[u8] =
    include_bytes!("../../../tests/fixtures/public-tombstone-blocked.json");
const PUBLIC_SUSPENDED: &[u8] =
    include_bytes!("../../../tests/fixtures/public-tombstone-suspended.json");
const PUBLIC_VIDEO_FALLBACK: &[u8] =
    include_bytes!("../../../tests/fixtures/public-video-fallback-lists.json");
const EMBED_IMAGE: &[u8] = include_bytes!("../../../tests/fixtures/embed-image.json");
const EMBED_VIDEO: &[u8] = include_bytes!("../../../tests/fixtures/embed-video.json");
const EMBED_TOMBSTONE: &[u8] = include_bytes!("../../../tests/fixtures/embed-tombstone.json");
const GATEWAY_BUNDLE: &[u8] = include_bytes!("../../../tests/fixtures/gateway-bundle.json");
const GATEWAY_RESTRICTED: &[u8] = include_bytes!("../../../tests/fixtures/gateway-restricted.json");
const MALFORMED: &[u8] = include_bytes!("../../../tests/fixtures/malformed.json");

fn options(gateway: Option<&str>) -> ResolutionOptions {
    ResolutionOptions {
        profile: RuntimeProfile::Browser,
        gateway_endpoint: gateway.map(str::to_owned),
    }
}

fn start(gateway: Option<&str>) -> ResolutionStep {
    start_resolution(INPUT_CANONICAL.trim(), options(gateway)).unwrap()
}

fn request(
    step: ResolutionStep,
) -> (
    media_resolver_core::ResolutionSession,
    media_resolver_core::PreparedRequest,
) {
    match step {
        ResolutionStep::Request {
            session, request, ..
        } => (session, *request),
        other => panic!("expected request step, got {other:?}"),
    }
}

fn failed_code(step: ResolutionStep) -> ResolveErrorCode {
    match step {
        ResolutionStep::Failed { error } => error.code,
        other => panic!("expected failed step, got {other:?}"),
    }
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
fn can_start_resolution_from_a_valid_source_key() {
    let step = start_resolution_from_key("2011590242474893548", options(None)).unwrap();
    let (_, request) = request(step);
    assert_eq!(request.route_key, "r0");
    assert!(request.url.contains("/2/status/2011590242474893548"));
}

#[test]
fn rejects_invalid_source_key_for_direct_start() {
    assert_eq!(
        start_resolution_from_key("not-a-key", options(None)),
        Err(ResolveError::InvalidInput)
    );
}

#[test]
fn public_route_is_first_and_resolves_images() {
    let (session, first) = request(start(None));
    assert_eq!(first.route_key, "r0");
    assert!(first.url.contains("/2/status/2011590242474893548"));
    let result = accept_response(session, 200, PUBLIC_IMAGE).unwrap();
    match result {
        ResolutionStep::Resolved { result } => {
            assert_eq!(result.resources.len(), 1);
            assert_eq!(result.resources[0].kind, ResourceKind::Image);
            assert!(result.resources[0].preferred_url.contains("name=orig"));
        }
        other => panic!("expected resolved step, got {other:?}"),
    }
}

#[test]
fn public_route_preserves_multiple_images() {
    let (session, _) = request(start(None));
    let result = accept_response(session, 200, PUBLIC_IMAGES).unwrap();
    match result {
        ResolutionStep::Resolved { result } => assert_eq!(result.resources.len(), 2),
        other => panic!("expected resolved step, got {other:?}"),
    }
}

#[test]
fn public_video_preserves_mp4_webm_hls_and_metadata() {
    let (session, _) = request(start(None));
    let result = accept_response(session, 200, PUBLIC_VIDEO).unwrap();
    match result {
        ResolutionStep::Resolved { result } => {
            let item = &result.resources[0];
            assert_eq!(item.kind, ResourceKind::Video);
            assert_eq!(item.duration_ms, Some(12_345));
            assert!(
                item.variants
                    .iter()
                    .any(|v| v.container.as_deref() == Some("mp4"))
            );
            assert!(
                item.variants
                    .iter()
                    .any(|v| v.container.as_deref() == Some("webm"))
            );
            assert!(
                item.variants
                    .iter()
                    .any(|v| v.container.as_deref() == Some("m3u8"))
            );
            assert_eq!(item.variants[0].container.as_deref(), Some("mp4"));
            assert_eq!(item.variants[0].codec.as_deref(), Some("h264"));
            assert_eq!(item.preferred_url, item.variants[0].url);
        }
        other => panic!("expected resolved step, got {other:?}"),
    }
}

#[test]
fn public_animation_is_normalized_as_animation() {
    let (session, _) = request(start(None));
    let result = accept_response(session, 200, PUBLIC_ANIMATION).unwrap();
    match result {
        ResolutionStep::Resolved { result } => {
            assert_eq!(result.resources[0].kind, ResourceKind::Animation);
            assert_eq!(
                result.resources[0].variants[0].container.as_deref(),
                Some("mp4")
            );
        }
        other => panic!("expected resolved step, got {other:?}"),
    }
}

#[test]
fn public_animation_transcode_is_normalized_as_animation() {
    let (session, _) = request(start(None));
    let result = accept_response(session, 200, PUBLIC_ANIMATION_TRANSCODE).unwrap();
    match result {
        ResolutionStep::Resolved { result } => {
            let item = &result.resources[0];
            assert_eq!(item.kind, ResourceKind::Animation);
            assert_eq!(item.variants[0].container.as_deref(), Some("mp4"));
            assert!(item.preferred_url.ends_with("animation.mp4"));
        }
        other => panic!("expected resolved step, got {other:?}"),
    }
}

#[test]
fn public_unavailable_falls_back_to_managed_gateway() {
    let (session, _) = request(start(Some("https://gateway.example.invalid")));
    let next = accept_response(session, 200, PUBLIC_UNAVAILABLE).unwrap();
    let (session, request) = request(next);
    assert_eq!(request.route_key, "r1");
    assert_eq!(
        request.url,
        "https://gateway.example.invalid/v1/resources/2011590242474893548"
    );
    let result = accept_response(session, 200, GATEWAY_BUNDLE).unwrap();
    assert!(matches!(result, ResolutionStep::Resolved { .. }));
}

#[test]
fn managed_unavailable_falls_back_to_legacy_route() {
    let (session, _) = request(start(Some("https://gateway.example.invalid")));
    let next = accept_response(session, 200, PUBLIC_UNAVAILABLE).unwrap();
    let (session, managed) = request(next);
    assert_eq!(managed.route_key, "r1");
    let next =
        accept_response(session, 200, br#"{"error":{"code":"remote_unavailable"}}"#).unwrap();
    let (session, legacy) = request(next);
    assert_eq!(legacy.route_key, "r2");
    let result = accept_response(session, 200, EMBED_IMAGE).unwrap();
    assert!(matches!(result, ResolutionStep::Resolved { .. }));
}

#[test]
fn managed_restricted_is_terminal() {
    let (session, _) = request(start(Some("https://gateway.example.invalid")));
    let next = accept_response(session, 200, PUBLIC_UNAVAILABLE).unwrap();
    let (session, _) = request(next);
    let result = accept_response(session, 200, GATEWAY_RESTRICTED).unwrap();
    assert_eq!(failed_code(result), ResolveErrorCode::RemoteRestricted);
}

#[test]
fn malformed_routes_aggregate_to_invalid_response() {
    let (session, _) = request(start(None));
    let next = accept_response(session, 200, MALFORMED).unwrap();
    let (session, _) = request(next);
    let result = accept_response(session, 200, MALFORMED).unwrap();
    assert_eq!(failed_code(result), ResolveErrorCode::InvalidResponse);
}

#[test]
fn public_private_is_terminal_and_does_not_fallback() {
    let (session, _) = request(start(Some("https://gateway.example.invalid")));
    let result = accept_response(session, 200, PUBLIC_PRIVATE).unwrap();
    assert_eq!(failed_code(result), ResolveErrorCode::RemoteRestricted);
}

#[test]
fn public_blocked_is_terminal_and_does_not_fallback() {
    let (session, _) = request(start(Some("https://gateway.example.invalid")));
    let result = accept_response(session, 200, PUBLIC_BLOCKED).unwrap();
    assert_eq!(failed_code(result), ResolveErrorCode::RemoteRestricted);
}

#[test]
fn public_suspended_is_terminal_unavailable() {
    let (session, _) = request(start(Some("https://gateway.example.invalid")));
    let result = accept_response(session, 200, PUBLIC_SUSPENDED).unwrap();
    assert_eq!(failed_code(result), ResolveErrorCode::RemoteUnavailable);
}

#[test]
fn public_falls_back_to_typed_media_lists_when_all_is_empty() {
    let (session, _) = request(start(None));
    let result = accept_response(session, 200, PUBLIC_VIDEO_FALLBACK).unwrap();
    match result {
        ResolutionStep::Resolved { result } => {
            assert_eq!(result.resources.len(), 1);
            assert_eq!(result.resources[0].kind, ResourceKind::Video);
            assert_eq!(
                result.resources[0].variants[0].container.as_deref(),
                Some("mp4")
            );
        }
        other => panic!("expected resolved step, got {other:?}"),
    }
}

#[test]
fn public_deleted_is_terminal_not_found() {
    let (session, _) = request(start(None));
    let result = accept_response(session, 200, PUBLIC_DELETED).unwrap();
    assert_eq!(failed_code(result), ResolveErrorCode::RemoteNotFound);
}

#[test]
fn access_blocked_advances_to_next_route() {
    let (session, _) = request(start(None));
    let next = accept_transport_failure(session, TransportFailure::AccessBlocked).unwrap();
    let (_, request) = request(next);
    assert_eq!(request.route_key, "r1");
}

#[test]
fn exhausted_rate_limit_falls_back() {
    let (session, _) = request(start(None));
    let next = accept_response(session, 429, b"{}").unwrap();
    let (_, request) = request(next);
    assert_eq!(request.route_key, "r1");
}

#[test]
fn legacy_tombstone_is_unavailable_not_not_found() {
    let (session, _) = request(start(None));
    let next = accept_response(session, 200, PUBLIC_UNAVAILABLE).unwrap();
    let (session, _) = request(next);
    let result = accept_response(session, 200, EMBED_TOMBSTONE).unwrap();
    assert_eq!(failed_code(result), ResolveErrorCode::RemoteUnavailable);
}

#[test]
fn all_not_found_aggregates_to_not_found() {
    let (session, _) = request(start(None));
    let next = accept_response(session, 404, b"{}").unwrap();
    let (session, _) = request(next);
    let result = accept_response(session, 404, b"{}").unwrap();
    assert_eq!(failed_code(result), ResolveErrorCode::RemoteNotFound);
}

#[test]
fn normal_document_without_media_wins_over_unavailable_fallback() {
    let (session, _) = request(start(None));
    let next = accept_response(session, 200, PUBLIC_NO_MEDIA).unwrap();
    let (session, _) = request(next);
    let result = accept_response(session, 200, EMBED_TOMBSTONE).unwrap();
    assert_eq!(failed_code(result), ResolveErrorCode::NoResources);
}

#[test]
fn legacy_video_is_still_supported_and_keeps_hls() {
    let (session, _) = request(start(None));
    let next = accept_response(session, 200, PUBLIC_UNAVAILABLE).unwrap();
    let (session, _) = request(next);
    let result = accept_response(session, 200, EMBED_VIDEO).unwrap();
    match result {
        ResolutionStep::Resolved { result } => {
            let item = &result.resources[0];
            assert!(
                item.variants
                    .iter()
                    .any(|v| v.container.as_deref() == Some("m3u8"))
            );
            assert_eq!(item.preferred_url, item.variants[0].url);
        }
        other => panic!("expected resolved step, got {other:?}"),
    }
}

#[test]
fn managed_gateway_allows_loopback_http_for_local_development() {
    let result = start_resolution(
        INPUT_CANONICAL.trim(),
        options(Some("http://127.0.0.1:8787")),
    );
    assert!(result.is_ok());
}

#[test]
fn managed_gateway_requires_a_safe_https_endpoint() {
    let result = start_resolution(
        INPUT_CANONICAL.trim(),
        options(Some("http://gateway.example.invalid?target=anything")),
    );
    assert_eq!(result, Err(ResolveError::Internal));
}

#[test]
fn managed_gateway_rejects_mismatched_source_key() {
    let (session, _) = request(start(Some("https://gateway.example.invalid")));
    let next = accept_response(session, 200, PUBLIC_UNAVAILABLE).unwrap();
    let (session, _) = request(next);
    let body = br#"{
        "schemaVersion": 1,
        "sourceKey": "999",
        "resources": [{
            "id": "r",
            "kind": "image",
            "preferredUrl": "https://assets.invalid/a.jpg",
            "variants": []
        }]
    }"#;
    let next = accept_response(session, 200, body).unwrap();
    let (_, request) = request(next);
    assert_eq!(request.route_key, "r2");
}

#[test]
fn serialized_steps_use_neutral_camel_case_request_fields() {
    let value = serde_json::to_value(start(None)).unwrap();
    assert_eq!(value["kind"], "request");
    assert!(value.get("sourceKey").is_some());
    assert!(value.get("normalizedInput").is_some());
    assert!(value["request"].get("routeKey").is_some());
    assert!(value["request"].get("retryPolicy").is_some());
}
