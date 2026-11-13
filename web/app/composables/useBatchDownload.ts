import type { ResourceItem } from "~/types/engine";
import type { ResolveTask } from "~/types/task";
import { zip } from "fflate";
import { resourceFilename } from "~/utils/resource-filename";
import { saveBlob } from "~/utils/save-blob";

export interface BatchEntry {
    sourceKey: string;
    resource: ResourceItem;
    resourceIndex: number;
    resourceCount: number;
}

export interface BatchGroup {
    id: string;
    title: string;
    subtitle?: string;
    entries: BatchEntry[];
}

export type BatchMode = "files" | "zip";

export type BatchStateName =
    | "idle"
    | "preparing"
    | "downloading"
    | "packaging"
    | "completed"
    | "failed"
    | "cancelled";

export interface BatchState {
    state: BatchStateName;
    total: number;
    completed: number;
    failed: number;
    current?: string;
    message?: string;
    mode?: BatchMode;
    archiveName?: string;
}

const IDLE: BatchState = {
    state: "idle",
    total: 0,
    completed: 0,
    failed: 0
};

let controller: AbortController | null = null;

export function batchEntriesForTask(task: ResolveTask): BatchEntry[] {
    const result = task.result;
    if (!result) {
        return [];
    }

    return result.resources.map((resource, resourceIndex) => ({
        sourceKey: result.sourceKey,
        resource,
        resourceIndex,
        resourceCount: result.resources.length
    }));
}

export function useBatchDownload() {
    const batch = useState<BatchState>("batch:downloads", () => ({ ...IDLE }));
    const downloads = useDownloadManager();

    const active = computed(() => ["preparing", "downloading", "packaging"].includes(
        batch.value.state
    ));

    const update = (patch: Partial<BatchState>) => {
        batch.value = { ...batch.value, ...patch };
    };

    const start = async (
        entries: BatchEntry[],
        mode: BatchMode,
        archiveName: string
    ): Promise<"completed" | "partial" | "failed" | "cancelled"> => {
        if (active.value || !entries.length) {
            return "cancelled";
        }

        controller = new AbortController();
        const signal = controller.signal;
        let completed = 0;
        let failed = 0;
        const archiveFiles: Array<{ name: string; data: Uint8Array }> = [];
        const usedNames = new Set<string>();

        update({
            state: "preparing",
            total: entries.length,
            completed: 0,
            failed: 0,
            current: undefined,
            message: undefined,
            mode,
            archiveName
        });

        try {
            for (const entry of entries) {
                if (signal.aborted) {
                    throw new DOMException("Aborted", "AbortError");
                }

                const filename = uniqueFilename(
                    resourceFilename(
                        entry.sourceKey,
                        entry.resource,
                        entry.resourceIndex,
                        entry.resourceCount
                    ),
                    usedNames
                );
                update({
                    state: "downloading",
                    current: filename
                });

                try {
                    const blob = await downloads.fetchResourceBlob(
                        entry.sourceKey,
                        entry.resource,
                        undefined,
                        undefined,
                        signal
                    );

                    if (mode === "zip") {
                        archiveFiles.push({
                            name: filename,
                            data: new Uint8Array(await blob.arrayBuffer())
                        });
                    } else {
                        saveBlob(blob, filename);
                    }
                    completed += 1;
                } catch (error) {
                    if (isAbort(error)) {
                        throw error;
                    }
                    failed += 1;
                }

                update({
                    completed,
                    failed
                });
            }

            if (mode === "zip") {
                if (!archiveFiles.length) {
                    update({
                        state: "failed",
                        completed,
                        failed,
                        current: undefined,
                        message: "No resources could be downloaded."
                    });
                    return "failed";
                }

                update({
                    state: "packaging",
                    current: archiveName
                });
                const archive = await createZip(archiveFiles);
                saveBlob(archive, archiveName);
            }

            update({
                state: failed && !completed
                    ? "failed"
                    : "completed",
                completed,
                failed,
                current: undefined
            });
            return failed
                ? "partial"
                : "completed";
        } catch (error) {
            if (isAbort(error)) {
                update({
                    state: "cancelled",
                    current: undefined
                });
                return "cancelled";
            }

            update({
                state: "failed",
                current: undefined,
                message: error instanceof Error
                    ? error.message
                    : "Batch download failed."
            });
            return "failed";
        } finally {
            controller = null;
        }
    };

    const cancel = () => {
        controller?.abort();
    };

    const reset = () => {
        if (!active.value) {
            batch.value = { ...IDLE };
        }
    };

    return { state: readonly(batch), active, start, cancel, reset };
}

function uniqueFilename(filename: string, used: Set<string>): string {
    if (!used.has(filename)) {
        used.add(filename);
        return filename;
    }

    const dot = filename.lastIndexOf(".");
    const base = dot > 0
        ? filename.slice(0, dot)
        : filename;
    const extension = dot > 0
        ? filename.slice(dot)
        : "";
    let sequence = 2;
    let candidate = `${base}-${sequence}${extension}`;
    while (used.has(candidate)) {
        sequence += 1;
        candidate = `${base}-${sequence}${extension}`;
    }
    used.add(candidate);
    return candidate;
}

function createZip(files: Array<{ name: string; data: Uint8Array }>): Promise<Blob> {
    return new Promise((resolve, reject) => {
        const input: Record<string, [Uint8Array, { level: 0 }]> = {};
        for (const file of files) {
            input[file.name] = [file.data, { level: 0 }];
        }

        zip(input, { level: 0 }, (error, data) => {
            if (error) {
                reject(error);
                return;
            }
            resolve(new Blob([data], { type: "application/zip" }));
        });
    });
}

function isAbort(error: unknown): boolean {
    return error instanceof DOMException && error.name === "AbortError";
}
