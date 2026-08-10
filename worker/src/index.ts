import { respond, start_key, transport_failed } from "../pkg/engine.js";

interface Env {
    ASSETS: {
        fetch(request: Request): Promise<Response>;
    };
}

interface RetryPolicy {
    maxRetries: number;
    delaysMs: number[];
    retryStatuses: number[];
}

interface PreparedRequest {
    routeKey: string;
    url: string;
    method: "GET";
    headers?: Array<{ name: string; value: string }>;
    retryPolicy: RetryPolicy;
}

type ResolutionSession = unknown;

type ResolutionStep =
        | {
    kind: "request";
    session: ResolutionSession;
    request: PreparedRequest;
    sourceKey: string;
    normalizedInput: string;
}
        | {
    kind: "resolved";
    result: ResourceBundle;
}
        | {
    kind: "failed";
    error: ResolveFailure;
};

interface ResolveFailure {
    code: string;
    message?: string;
}

interface ResourceVariant {
    url: string;
    mimeType?: string;
    container?: string;
    codec?: string;
    bitrate?: number;
    sizeBytes?: number;
    width?: number;
    height?: number;
}

interface ResourceItem {
    id: string;
    kind: "image" | "video" | "animation" | "unknown";
    preferredUrl: string;
    previewUrl?: string;
    width?: number;
    height?: number;
    durationMs?: number;
    variants: ResourceVariant[];
}

interface ResourceBundle {
    schemaVersion: number;
    sourceKey: string;
    resources: ResourceItem[];
}

type TransportResult =
        | { kind: "response"; status: number; body: Uint8Array }
        | { kind: "network" | "timeout" };

const RESOLVE_TIMEOUT_MS = 15_000;
const DOWNLOAD_CONNECT_TIMEOUT_MS = 20_000;
const MAX_ENGINE_STEPS = 8;

export default {
    async fetch(request: Request, env: Env): Promise<Response> {
        const url = new URL(request.url);

        if (url.pathname.startsWith("/v1/")) {
            return handleApi(request, url);
        }

        return env.ASSETS.fetch(request);
    }
};

async function handleApi(request: Request, url: URL): Promise<Response> {
    if (request.method !== "GET") {
        return apiError(405, "method_not_allowed", "Only GET is supported.", {
            Allow: "GET"
        });
    }

    const resourceMatch = /^\/v1\/resources\/([0-9]{2,20})\/?$/.exec(url.pathname);
    if (resourceMatch) {
        if (url.search) {
            return apiError(400, "invalid_request", "Query parameters are not supported here.");
        }
        return handleResolve(resourceMatch[1]);
    }

    const downloadMatch = /^\/v1\/download\/([0-9]{2,20})\/([^/]+)\/?$/.exec(url.pathname);
    if (downloadMatch) {
        let resourceId: string;
        try {
            resourceId = decodeURIComponent(downloadMatch[2]);
        } catch {
            return apiError(400, "invalid_request", "The resource identifier is invalid.");
        }
        if (!resourceId || resourceId.length > 256) {
            return apiError(400, "invalid_request", "The resource identifier is invalid.");
        }
        for (const key of url.searchParams.keys()) {
            if (key !== "variant") {
                return apiError(400, "invalid_request", "Unsupported query parameter.");
            }
        }
        const variant = parseVariantIndex(url.searchParams.get("variant"));
        if (variant === null) {
            return apiError(400, "invalid_request", "The variant index is invalid.");
        }
        return handleDownload(downloadMatch[1], resourceId, variant);
    }

    return apiError(404, "not_found", "API route not found.");
}

async function handleResolve(sourceKey: string): Promise<Response> {
    const resolved = await resolveSource(sourceKey);
    if ("error" in resolved) {
        return failureResponse(resolved.error);
    }

    return jsonResponse(resolved.result, 200, {
        "Cache-Control": "no-store"
    });
}

async function handleDownload(
        sourceKey: string,
        resourceId: string,
        variantIndex: number | undefined
): Promise<Response> {
    const resolved = await resolveSource(sourceKey);
    if ("error" in resolved) {
        return failureResponse(resolved.error);
    }

    const resourceIndex = resolved.result.resources.findIndex(item => item.id === resourceId);
    if (resourceIndex < 0) {
        return apiError(404, "resource_not_found", "Resource not found.");
    }

    const resource = resolved.result.resources[resourceIndex];
    const variant = variantIndex === undefined
            ? undefined
            : resource.variants[variantIndex];
    if (variantIndex !== undefined && !variant) {
        return apiError(404, "variant_not_found", "Variant not found.");
    }

    const target = variant?.url || resource.preferredUrl;
    if (!isSafeResourceUrl(target)) {
        return apiError(502, "invalid_response", "Resolved resource address is invalid.");
    }

    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), DOWNLOAD_CONNECT_TIMEOUT_MS);
    let upstream: Response;
    try {
        upstream = await fetch(target, {
            method: "GET",
            redirect: "follow",
            signal: controller.signal
        });
    } catch {
        return apiError(
                502,
                "network_unavailable",
                "Resource download is temporarily unavailable."
        );
    } finally {
        clearTimeout(timer);
    }

    if (!upstream.ok || !upstream.body) {
        return apiError(
                upstream.status === 404
                        ? 404
                        : 502,
                upstream.status === 404
                        ? "resource_not_found"
                        : "remote_unavailable",
                "Resource download is temporarily unavailable."
        );
    }

    const contentType = upstream.headers.get("content-type")
            || variant?.mimeType
            || "application/octet-stream";
    const filename = downloadFilename(resource, resourceIndex, variant, contentType, target);
    const headers = new Headers({
        "Cache-Control": "no-store",
        "Content-Disposition": `attachment; filename="${filename}"`,
        "Content-Type": contentType,
        "X-Content-Type-Options": "nosniff"
    });

    return new Response(upstream.body, {
        status: 200,
        headers
    });
}

async function resolveSource(
        sourceKey: string
): Promise<{ result: ResourceBundle } | { error: ResolveFailure }> {
    let step: ResolutionStep;
    try {
        step = start_key(sourceKey, { profile: "native" }) as ResolutionStep;
    } catch (error) {
        return { error: normalizeEngineError(error) };
    }

    for (let index = 0; index < MAX_ENGINE_STEPS; index += 1) {
        if (step.kind === "resolved") {
            return { result: step.result };
        }
        if (step.kind === "failed") {
            return { error: step.error };
        }

        const transport = await executePreparedRequest(step.request);
        try {
            step = transport.kind === "response"
                    ? respond(step.session, transport.status, transport.body) as ResolutionStep
                    : transport_failed(step.session, transport.kind) as ResolutionStep;
        } catch (error) {
            return { error: normalizeEngineError(error) };
        }
    }

    return {
        error: {
            code: "internal",
            message: "Resolution exceeded the maximum number of steps."
        }
    };
}

async function executePreparedRequest(request: PreparedRequest): Promise<TransportResult> {
    const retries = Math.max(0, Math.min(4, request.retryPolicy?.maxRetries ?? 0));
    const retryStatuses = new Set(request.retryPolicy?.retryStatuses ?? []);

    for (let attempt = 0; attempt <= retries; attempt += 1) {
        const controller = new AbortController();
        const timer = setTimeout(() => controller.abort(), RESOLVE_TIMEOUT_MS);
        try {
            const headers = new Headers();
            for (const header of request.headers || []) {
                headers.set(header.name, header.value);
            }
            const response = await fetch(request.url, {
                method: request.method,
                headers,
                redirect: "follow",
                signal: controller.signal
            });
            const body = new Uint8Array(await response.arrayBuffer());

            if (attempt < retries && retryStatuses.has(response.status)) {
                await retryDelay(request.retryPolicy, attempt, response.headers.get("retry-after"));
                continue;
            }

            return {
                kind: "response",
                status: response.status,
                body
            };
        } catch (error) {
            if (attempt < retries) {
                await retryDelay(request.retryPolicy, attempt);
                continue;
            }
            if (error instanceof DOMException && error.name === "AbortError") {
                return { kind: "timeout" };
            }
            return { kind: "network" };
        } finally {
            clearTimeout(timer);
        }
    }

    return { kind: "network" };
}

async function retryDelay(
        policy: RetryPolicy,
        attempt: number,
        retryAfter?: string | null
) {
    const retryAfterMs = parseRetryAfter(retryAfter);
    const configured = policy.delaysMs?.[attempt] ?? 0;
    const delayMs = Math.min(10_000, Math.max(retryAfterMs, configured, 0));
    if (delayMs > 0) {
        await new Promise(resolve => setTimeout(resolve, delayMs));
    }
}

function parseRetryAfter(value?: string | null): number {
    if (!value) {
        return 0;
    }
    const seconds = Number(value);
    if (Number.isFinite(seconds) && seconds >= 0) {
        return Math.round(seconds * 1000);
    }
    const at = Date.parse(value);
    return Number.isFinite(at)
            ? Math.max(0, at - Date.now())
            : 0;
}

function parseVariantIndex(value: string | null): number | undefined | null {
    if (value === null || value === "") {
        return undefined;
    }
    if (!/^\d+$/.test(value)) {
        return null;
    }
    const index = Number(value);
    return Number.isSafeInteger(index) && index >= 0 && index <= 10_000
            ? index
            : null;
}

function isSafeResourceUrl(value: string): boolean {
    try {
        const url = new URL(value);
        return url.protocol === "https:" && !url.username && !url.password;
    } catch {
        return false;
    }
}

function downloadFilename(
        resource: ResourceItem,
        resourceIndex: number,
        variant: ResourceVariant | undefined,
        contentType: string,
        target: string
): string {
    const width = variant?.width || resource.width;
    const height = variant?.height || resource.height;
    const dimensions = width && height
            ? `-${width}x${height}`
            : "";
    const animation = resource.kind === "animation"
            ? "-animation"
            : "";
    const extension = extensionFor(variant?.container, contentType, target);
    return `resource-${resourceIndex + 1}${dimensions}${animation}.${extension}`;
}

function extensionFor(container: string | undefined, contentType: string, target: string): string {
    const normalized = container?.toLowerCase();
    if (normalized && /^[a-z0-9]{2,8}$/.test(normalized)) {
        return normalized === "m3u8"
                ? "m3u8"
                : normalized;
    }

    const mime = contentType.split(";", 1)[0].trim().toLowerCase();
    const byMime: Record<string, string> = {
        "image/jpeg": "jpg",
        "image/png": "png",
        "image/webp": "webp",
        "image/gif": "gif",
        "video/mp4": "mp4",
        "video/webm": "webm",
        "application/vnd.apple.mpegurl": "m3u8",
        "application/x-mpegurl": "m3u8"
    };
    if (byMime[mime]) {
        return byMime[mime];
    }

    try {
        const match = new URL(target).pathname.match(/\.([a-zA-Z0-9]{2,8})$/);
        if (match) {
            return match[1].toLowerCase();
        }
    } catch {
        // The URL was already validated before this helper is called.
    }
    return "bin";
}

function normalizeEngineError(error: unknown): ResolveFailure {
    if (error && typeof error === "object") {
        const value = error as { code?: unknown; message?: unknown };
        return {
            code: typeof value.code === "string"
                    ? value.code
                    : "internal",
            message: typeof value.message === "string"
                    ? value.message
                    : undefined
        };
    }
    return { code: "internal" };
}

function failureResponse(error: ResolveFailure): Response {
    const status = statusForError(error.code);
    return jsonResponse({ error }, status, {
        "Cache-Control": "no-store"
    });
}

function statusForError(code: string): number {
    switch (code) {
        case "invalid_input":
        case "unsupported_input":
            return 400;
        case "remote_restricted":
            return 403;
        case "remote_not_found":
        case "no_resources":
            return 404;
        case "rate_limited":
            return 429;
        case "network_unavailable":
        case "remote_unavailable":
            return 503;
        case "remote_rejected":
            return 502;
        default:
            return 502;
    }
}

function apiError(
        status: number,
        code: string,
        message: string,
        extraHeaders: HeadersInit = {}
): Response {
    const headers = new Headers(extraHeaders);
    headers.set("Cache-Control", "no-store");
    return jsonResponse({ error: { code, message } }, status, headers);
}

function jsonResponse(value: unknown, status: number, extraHeaders: HeadersInit = {}): Response {
    const headers = new Headers(extraHeaders);
    headers.set("Content-Type", "application/json; charset=utf-8");
    headers.set("X-Content-Type-Options", "nosniff");
    return new Response(JSON.stringify(value), {
        status,
        headers
    });
}
