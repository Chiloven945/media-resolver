import type { ResourceItem, ResourceVariant } from "~/types/engine";

export function resourceKindLabel(kind: ResourceItem["kind"]): string {
    return {
        image: "Image",
        video: "Video",
        animation: "Animation",
        unknown: "Resource"
    }[kind];
}

export function displayDimensions(width?: number, height?: number): string {
    return width && height
            ? `${width}×${height}`
            : "Original size";
}

export function displayFormat(variant?: ResourceVariant): string {
    if (variant?.container) {
        return variant.container.toUpperCase();
    }

    return variant?.mimeType?.split("/").pop()?.toUpperCase() || "Unknown";
}

export function displayCodec(codec?: string): string {
    switch (codec?.toLowerCase()) {
        case "h264":
            return "H.264";
        case "hevc":
            return "HEVC";
        case "vp9":
            return "VP9";
        case "av1":
            return "AV1";
        default:
            return codec?.toUpperCase() || "—";
    }
}

export function displayBitrate(bitrate?: number): string {
    if (!bitrate || bitrate <= 0) {
        return "—";
    }

    if (bitrate >= 1_000_000) {
        return `${(
                bitrate / 1_000_000
        ).toFixed(1)} Mbps`;
    }

    return `${Math.round(bitrate / 1_000)} Kbps`;
}

export function displayBytes(bytes?: number): string {
    if (!bytes || bytes <= 0) {
        return "—";
    }

    const units = ["B", "KB", "MB", "GB"];
    let value = bytes;
    let index = 0;
    while (value >= 1024 && index < units.length - 1) {
        value /= 1024;
        index += 1;
    }

    return `${value.toFixed(index === 0
            ? 0
            : 1)} ${units[index]}`;
}

export function formatSummary(variant?: ResourceVariant): string {
    if (!variant) {
        return "";
    }

    return [
        displayFormat(variant),
        variant.codec
                ? displayCodec(variant.codec)
                : "",
        variant.bitrate
                ? displayBitrate(variant.bitrate)
                : ""
    ].filter(Boolean).join(" · ");
}

export function isStreamVariant(variant?: ResourceVariant, url?: string): boolean {
    const container = variant?.container?.toLowerCase();
    const mime = variant?.mimeType?.toLowerCase();
    return container === "m3u8"
            || mime === "application/x-mpegurl"
            || mime === "application/vnd.apple.mpegurl"
            || Boolean(url?.toLowerCase().includes(".m3u8"));
}
