import type { ResourceItem, ResourceVariant } from "../types/engine";

export function resourceFilename(
        resource: ResourceItem,
        resourceIndex: number,
        variant?: ResourceVariant
): string {
    const selected = variant || resource.variants.find(candidate => candidate.url
            === resource.preferredUrl);
    const extension = extensionFor(resource, selected);
    const base = `resource-${resourceIndex + 1}`;

    if (resource.kind === "animation") {
        return `${base}-animation.${extension}`;
    }

    if ((
                    resource.kind === "video" || resource.kind === "unknown"
            )
            && selected?.width && selected?.height) {
        return `${base}-${selected.width}x${selected.height}.${extension}`;
    }

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

    if (resource.kind === "image") {
        return "jpg";
    }
    if (resource.kind === "video" || resource.kind === "animation") {
        return "mp4";
    }
    return "bin";
}
