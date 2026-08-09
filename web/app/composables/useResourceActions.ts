import type { ResourceItem, ResourceVariant } from "~/types/engine";
import { resourceFilename } from "~/utils/resource-filename";

export function useResourceActions() {
    const open = (url: string) => {
        const parsed = requireHttps(url);
        window.open(parsed.toString(), "_blank", "noopener,noreferrer");
    };

    const filename = (
            resource: ResourceItem,
            resourceIndex: number,
            variant?: ResourceVariant
    ) => resourceFilename(resource, resourceIndex, variant);

    return { open, filename };
}

export function requireHttps(url: string): URL {
    const parsed = new URL(url);
    if (parsed.protocol !== "https:") {
        throw new Error("unsupported resource protocol");
    }
    return parsed;
}
