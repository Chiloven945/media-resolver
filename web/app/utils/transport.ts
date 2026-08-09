import type { PreparedRequest } from "~/types/resolution";

export type TransportResult =
        | {
    kind: "response";
    status: number;
    body: Uint8Array;
    retryAfterMs?: number;
}
        | { kind: "network_error" }
        | { kind: "access_blocked" }
        | { kind: "timeout" };

const REQUEST_TIMEOUT_MS = 20_000;

export async function executeRequest(
        request: PreparedRequest,
        signal: AbortSignal
): Promise<TransportResult> {
    const controller = new AbortController();
    let timedOut = false;
    const onAbort = () => controller.abort(signal.reason);
    signal.addEventListener("abort", onAbort, { once: true });
    const timer = window.setTimeout(() => {
        timedOut = true;
        controller.abort();
    }, REQUEST_TIMEOUT_MS);

    try {
        const headers = new Headers();
        for (const header of request.headers || []) {
            headers.set(header.name, header.value);
        }
        const response = await fetch(request.url, {
            method: request.method,
            headers,
            signal: controller.signal,
            credentials: "omit",
            redirect: "follow",
            referrerPolicy: "no-referrer"
        });
        const body = new Uint8Array(await response.arrayBuffer());
        return {
            kind: "response",
            status: response.status,
            body,
            retryAfterMs: parseRetryAfter(response.headers.get("retry-after"))
        };
    } catch (error) {
        if (signal.aborted) {
            throw new DOMException("Aborted", "AbortError");
        }
        if (timedOut) {
            return { kind: "timeout" };
        }
        if (error instanceof TypeError && navigator.onLine) {
            return { kind: "access_blocked" };
        }
        return { kind: "network_error" };
    } finally {
        window.clearTimeout(timer);
        signal.removeEventListener("abort", onAbort);
    }
}

function parseRetryAfter(value: string | null): number | undefined {
    if (!value) {
        return undefined;
    }
    const seconds = Number(value);
    if (Number.isFinite(seconds) && seconds >= 0) {
        return Math.min(seconds * 1000, 5 * 60 * 1000);
    }
    const timestamp = Date.parse(value);
    if (!Number.isNaN(timestamp)) {
        return Math.min(Math.max(timestamp - Date.now(), 0), 5 * 60 * 1000);
    }
    return undefined;
}
