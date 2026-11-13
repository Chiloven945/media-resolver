import type { ResourceItem, ResourceVariant } from "../types/engine";

export function resourceFilename(
    sourceKey: string,
    resource: ResourceItem,
    resourceIndex: number,
    resourceCount: number,
    variant?: ResourceVariant
): string {
    const selected = variant || resource.variants.find(candidate => candidate.url
        === resource.preferredUrl);
    const extension = extensionFor(resource, selected);
    const base = resourceCount > 1
        ? `${sourceKey}-${resourceIndex + 1}`
        : sourceKey;

    return `${base}.${extension}`;
}

function extensionFor(resource: ResourceItem, variant?: ResourceVariant): string {
    const container = variant?.container?.toLowerCase();
    if (container && /^[a-z0-9]{2,8}$/.test(container)) {
        return container === "jpeg"
            ? "jpg"
            : container;
    }

    const mime = variant?.mimeType?.toLowerCase();
    const byMime: Record<string, string> = {
        "image/jpeg": "jpg",
        "image/png": "png",
        "image/webp": "webp",
        "image/gif": "gif",
        "video/mp4": "mp4",
        "video/webm": "webm",
        "application/x-mpegurl": "m3u8",
        "application/vnd.apple.mpegurl": "m3u8"
    };
    if (mime && byMime[mime]) {
        return byMime[mime];
    }

    try {
        const path = new URL(variant?.url || resource.preferredUrl).pathname;
        const match = path.match(/\.([a-z0-9]{2,8})$/i);
        if (match?.[1]) {
            return match[1].toLowerCase() === "jpeg"
                ? "jpg"
                : match[1].toLowerCase();
        }
    } catch {
        // Use the type-derived fallback below.
    }

    switch (resource.kind) {
        case "image":
            return "jpg";

        case "video":
        case "animation":
            return "mp4";

        default:
            return "bin";
    }
}
