import type { ResourceItem, ResourceVariant } from "~/types/engine";
import type { DownloadResult, ResourceDownload } from "~/types/download";
import { requireHttps } from "~/composables/useResourceActions";
import { resourceFilename } from "~/utils/resource-filename";
import { buildManagedDownloadUrl } from "~/utils/download";
import { resolverEndpoint } from "~/utils/resolver-endpoint";

interface DownloadOptions {
    resourceIndex: number;
    resourceCount: number;
    variant?: ResourceVariant;
    variantIndex?: number;
}

const controllers = new Map<string, AbortController>();
const IDLE: ResourceDownload = { state: "idle" };

export function useDownloadManager() {
    const downloads = useState<Record<string, ResourceDownload>>(
        "downloads:state",
        () => (
            {}
        )
    );
    const config = useRuntimeConfig();

    const keyFor = (
        sourceKey: string,
        resourceId: string,
        variantIndex?: number
    ) => `${sourceKey}\u0000${resourceId}\u0000${variantIndex ?? "preferred"}`;

    const stateFor = (key: string): ResourceDownload => downloads.value[key] || IDLE;
    const progressFor = (key: string): number | undefined => stateFor(key).progress;

    const cancel = (key: string) => {
        controllers.get(key)?.abort();
        controllers.delete(key);
        delete downloads.value[key];
    };

    const download = async (
        sourceKey: string,
        resource: ResourceItem,
        options: DownloadOptions
    ): Promise<DownloadResult> => {
        const selected = options.variant;
        const url = requireHttps(selected?.url || resource.preferredUrl).toString();
        const key = keyFor(sourceKey, resource.id, options.variantIndex);
        if (["preparing", "downloading"].includes(stateFor(key).state)) {
            return "cancelled";
        }

        controllers.get(key)?.abort();
        const controller = new AbortController();
        controllers.set(key, controller);
        setState(key, { state: "preparing" });

        const fallbackName = resourceFilename(
            sourceKey,
            resource,
            options.resourceIndex,
            options.resourceCount,
            selected
        );
        try {
            const direct = await fetchBlob(url, controller.signal, progress => {
                setState(key, { state: "downloading", progress });
            });
            saveBlob(direct, fallbackName);
            setState(key, { state: "completed", progress: 100 });
            return "downloaded";
        } catch (error) {
            if (isAbort(error)) {
                delete downloads.value[key];
                return "cancelled";
            }
        }

        const gateway = buildManagedDownloadUrl(
            resolverEndpoint(config.public.resolverEndpoint) || "",
            sourceKey,
            resource.id,
            options.variantIndex
        );
        if (gateway) {
            try {
                setState(key, { state: "preparing" });
                const proxied = await fetchBlob(gateway, controller.signal, progress => {
                    setState(key, { state: "downloading", progress });
                });
                saveBlob(proxied, fallbackName);
                setState(key, { state: "completed", progress: 100 });
                return "downloaded";
            } catch (error) {
                if (isAbort(error)) {
                    delete downloads.value[key];
                    return "cancelled";
                }
            }
        }

        setState(key, { state: "failed", fallbackUrl: url });
        return "failed";
    };

    function setState(key: string, value: ResourceDownload) {
        downloads.value[key] = value;
    }

    return { keyFor, stateFor, progressFor, download, cancel };
}

async function fetchBlob(
    url: string,
    signal: AbortSignal,
    onProgress: (progress?: number) => void
): Promise<Blob> {
    const response = await fetch(url, {
        method: "GET",
        mode: "cors",
        credentials: "omit",
        referrerPolicy: "no-referrer",
        signal
    });
    if (!response.ok) {
        throw new Error(`download request failed with ${response.status}`);
    }

    const contentType = response.headers.get("content-type") || "application/octet-stream";
    const totalHeader = Number(response.headers.get("content-length") || 0);
    const total = Number.isFinite(totalHeader) && totalHeader > 0
        ? totalHeader
        : undefined;

    if (!response.body) {
        const blob = await response.blob();
        onProgress(100);
        return blob;
    }

    const reader = response.body.getReader();
    const chunks: BlobPart[] = [];
    let received = 0;
    onProgress(total
        ? 0
        : undefined);

    while (true) {
        const { done, value } = await reader.read();
        if (done) {
            break;
        }
        if (value) {
            chunks.push(value);
            received += value.byteLength;
            onProgress(total
                ? Math.min(
                    100,
                    Math.round((
                        received / total
                    ) * 100)
                )
                : undefined);
        }
    }

    return new Blob(chunks, { type: contentType });
}

function saveBlob(blob: Blob, filename: string) {
    const objectUrl = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = objectUrl;
    anchor.download = sanitizeFilename(filename) || "resource";
    anchor.rel = "noopener noreferrer";
    anchor.style.display = "none";
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    window.setTimeout(() => URL.revokeObjectURL(objectUrl), 1000);
}

function sanitizeFilename(value: string): string {
    return value
        .replace(/[\\/:*?"<>|\u0000-\u001F]/g, "-")
        .replace(/\s+/g, " ")
        .trim()
        .slice(0, 180);
}

function isAbort(error: unknown): boolean {
    return error instanceof DOMException && error.name === "AbortError";
}
