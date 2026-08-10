const SAME_ORIGIN = "same-origin";

export function resolverEndpoint(configured: unknown): string | undefined {
    const explicit = String(configured ?? "").trim();
    if (explicit === SAME_ORIGIN) {
        return runtimeOrigin();
    }
    if (explicit) {
        return normalizeResolverEndpoint(explicit);
    }
    return undefined;
}

export function normalizeResolverEndpoint(value: string): string | undefined {
    try {
        const url = new URL(value);
        if (
                !isAllowedResolverProtocol(url)
                || url.username
                || url.password
                || url.search
                || url.hash
        ) {
            return undefined;
        }
        url.pathname = url.pathname.replace(/\/+$/, "");
        return url.toString().replace(/\/$/, "");
    } catch {
        return undefined;
    }
}

export function isAllowedResolverProtocol(url: URL): boolean {
    if (url.protocol === "https:") {
        return true;
    }
    if (url.protocol !== "http:") {
        return false;
    }

    return ["localhost", "127.0.0.1", "[::1]"].includes(url.hostname);
}

function runtimeOrigin(): string | undefined {
    if (typeof window === "undefined") {
        return undefined;
    }
    return normalizeResolverEndpoint(window.location.origin);
}
