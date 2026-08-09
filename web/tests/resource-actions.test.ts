import { describe, expect, it } from "vitest";
import type { ResourceItem } from "../app/types/engine";
import { resourceFilename } from "../app/utils/resource-filename";
import { buildManagedDownloadUrl } from "../app/utils/download";

const image: ResourceItem = {
    id: "a",
    kind: "image",
    preferredUrl: "https://assets.example/image",
    width: 1200,
    height: 800,
    variants: [
        {
            url: "https://assets.example/image",
            mimeType: "image/jpeg",
            width: 1200,
            height: 800
        }
    ]
};

const video: ResourceItem = {
    id: "b",
    kind: "video",
    preferredUrl: "https://assets.example/video",
    width: 720,
    height: 1280,
    variants: [
        {
            url: "https://assets.example/video",
            mimeType: "video/mp4",
            container: "mp4",
            codec: "h264",
            width: 720,
            height: 1280
        }
    ]
};

describe("resource filenames", () => {
    it("uses a stable image filename", () => {
        expect(resourceFilename(image, 0, image.variants[0])).toBe("resource-1.jpg");
    });

    it("includes selected video dimensions", () => {
        expect(resourceFilename(video, 1, video.variants[0])).toBe("resource-2-720x1280.mp4");
    });

    it("marks animated video representations without pretending they are gif files", () => {
        const animation: ResourceItem = { ...video, kind: "animation" };
        expect(resourceFilename(animation, 2, animation.variants[0]))
                .toBe("resource-3-animation.mp4");
    });
});

describe("managed download URL", () => {
    it("uses only the configured endpoint, source key, resource id and variant index", () => {
        expect(buildManagedDownloadUrl("https://resolver.example/base", "123", "item:1", 2))
                .toBe("https://resolver.example/base/v1/download/123/item%3A1?variant=2");
    });

    it("rejects non-HTTPS and pre-parameterized endpoints", () => {
        expect(buildManagedDownloadUrl("http://resolver.example", "123", "item", 0))
                .toBeUndefined();
        expect(buildManagedDownloadUrl("https://resolver.example?target=x", "123", "item", 0))
                .toBeUndefined();
    });
});
