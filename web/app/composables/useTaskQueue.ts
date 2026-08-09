import type { ResourceBundle } from "~/types/engine";
import type { ResolveTask, TaskError } from "~/types/task";

interface CacheEntry {
    expiresAt: number;
    result: ResourceBundle;
}

export interface AddResult {
    status: "added" | "duplicate" | "invalid";
    task?: ResolveTask;
    errorCode?: string;
}

export interface AddManyResult {
    added: number;
    duplicates: number;
    invalid: number;
    firstTask?: ResolveTask;
}

export interface CompleteFromResponseResult {
    status: "ready" | "invalid" | "missing";
    errorCode?: string;
}

const CACHE_TTL_MS = 5 * 60 * 1000;
const cache = new Map<string, CacheEntry>();
const controllers = new Map<string, AbortController>();
let settingsWatcherInstalled = false;

export function useTaskQueue() {
    const tasks = useState<ResolveTask[]>("tasks:list", () => []);
    const activeCount = useState<number>("tasks:active", () => 0);
    const sequence = useState<number>("tasks:sequence", () => 0);
    const { settings } = useAppSettings();
    const { select } = useTaskSelection();
    const executor = useResolverExecutor();

    const queuedCount = computed(() => tasks.value.filter(task => task.state === "queued").length);
    const completedCount = computed(() => tasks.value.filter(task =>
            task.state === "ready" || task.state === "failed" || task.state === "cancelled"
    ).length);
    const totalCount = computed(() => tasks.value.length);

    const add = async (
            input: string,
            shouldSelect = settings.value.autoSelect
    ): Promise<AddResult> => {
        const value = input.trim();
        if (!value) {
            return { status: "invalid" };
        }

        let inspected;
        try {
            inspected = executor.inspect(value);
        } catch (error) {
            return { status: "invalid", errorCode: executor.errorCode(error) };
        }

        const duplicate = tasks.value.find(task =>
                task.sourceKey === inspected.sourceKey
                && ["queued", "connecting", "processing", "ready"].includes(task.state)
        );
        if (duplicate) {
            if (shouldSelect) {
                select(duplicate.id);
            }
            return { status: "duplicate", task: duplicate };
        }

        sequence.value += 1;
        const cached = getCached(inspected.sourceKey);
        const task: ResolveTask = {
            id: crypto.randomUUID(),
            sequence: sequence.value,
            input: value,
            sourceKey: inspected.sourceKey,
            normalizedInput: inspected.normalizedInput,
            state: cached
                    ? "ready"
                    : "queued",
            createdAt: Date.now(),
            completedAt: cached
                    ? Date.now()
                    : undefined,
            result: cached
        };

        tasks.value.push(task);
        if (shouldSelect) {
            select(task.id);
        }
        if (!cached) {
            pumpQueue();
        }
        return { status: "added", task };
    };

    const addMany = async (inputs: string[]): Promise<AddManyResult> => {
        let added = 0;
        let duplicates = 0;
        let invalid = 0;
        let firstTask: ResolveTask | undefined;

        for (const input of inputs) {
            const result = await add(input, false);
            if (result.status === "added") {
                added += 1;
                firstTask ||= result.task;
            } else if (result.status === "duplicate") {
                duplicates += 1;
                firstTask ||= result.task;
            } else {
                invalid += 1;
            }
        }

        if (firstTask && settings.value.autoSelect) {
            select(firstTask.id);
        }
        return { added, duplicates, invalid, firstTask };
    };

    const retry = (id: string) => {
        const task = findTask(id);
        if (!task) {
            return;
        }
        controllers.get(task.id)?.abort();
        controllers.delete(task.id);
        executor.clearRecovery(task.id);
        if (task.sourceKey) {
            cache.delete(task.sourceKey);
        }
        task.state = "queued";
        task.startedAt = undefined;
        task.completedAt = undefined;
        task.result = undefined;
        task.error = undefined;
        pumpQueue();
    };

    const cancel = (id: string) => {
        const task = findTask(id);
        if (!task) {
            return;
        }
        if (task.state === "queued") {
            task.state = "cancelled";
            task.completedAt = Date.now();
            return;
        }
        if (task.state === "connecting" || task.state === "processing") {
            controllers.get(task.id)?.abort();
            task.state = "cancelled";
            task.completedAt = Date.now();
        }
    };

    const remove = (id: string) => {
        const index = tasks.value.findIndex(task => task.id === id);
        if (index < 0) {
            return;
        }
        controllers.get(id)?.abort();
        controllers.delete(id);
        executor.clearRecovery(id);
        tasks.value.splice(index, 1);
        const selection = useTaskSelection();
        if (selection.selectedId.value === id) {
            selection.select(tasks.value[index]?.id || tasks.value[index - 1]?.id || null);
        }
    };

    const clearCompleted = () => {
        const removable = new Set(
                tasks.value
                        .filter(task => ["ready", "failed", "cancelled"].includes(task.state))
                        .map(task => task.id)
        );
        for (const id of removable) {
            executor.clearRecovery(id);
        }
        tasks.value = tasks.value.filter(task => !removable.has(task.id));
        const selection = useTaskSelection();
        if (selection.selectedId.value && removable.has(selection.selectedId.value)) {
            selection.select(tasks.value[0]?.id || null);
        }
    };

    const clearAll = () => {
        for (const task of tasks.value) {
            controllers.get(task.id)?.abort();
            executor.clearRecovery(task.id);
        }
        controllers.clear();
        tasks.value = [];
        select(null);
    };

    const canRecover = (id: string): boolean => executor.hasRecovery(id);

    const openResponse = (id: string): boolean => executor.openRecoveryResponse(id);

    const completeFromResponse = async (
            id: string,
            body: string
    ): Promise<CompleteFromResponseResult> => {
        const task = findTask(id);
        if (!task) {
            return { status: "missing" };
        }

        const value = body.trim();
        if (!value) {
            return { status: "invalid", errorCode: "invalid_response" };
        }

        controllers.get(task.id)?.abort();
        controllers.delete(task.id);
        task.state = "processing";
        task.error = undefined;
        task.result = undefined;
        task.startedAt ||= Date.now();
        task.completedAt = undefined;

        try {
            const result = executor.continueFromRecovery(task.id, value);
            task.sourceKey = result.sourceKey || task.sourceKey;
            task.result = result;
            task.state = "ready";
            task.completedAt = Date.now();
            if (task.sourceKey) {
                setCached(task.sourceKey, result);
            }
            return { status: "ready" };
        } catch (error) {
            const code = executor.errorCode(error) || "invalid_response";
            task.error = { code } satisfies TaskError;
            task.state = "failed";
            task.completedAt = Date.now();
            return { status: "invalid", errorCode: code };
        }
    };

    function findTask(id: string) {
        return tasks.value.find(task => task.id === id);
    }

    function getCached(key: string): ResourceBundle | undefined {
        const entry = cache.get(key);
        if (!entry) {
            return undefined;
        }
        if (entry.expiresAt <= Date.now()) {
            cache.delete(key);
            return undefined;
        }
        return entry.result;
    }

    function setCached(key: string, result: ResourceBundle) {
        cache.set(key, { expiresAt: Date.now() + CACHE_TTL_MS, result });
    }

    function pumpQueue() {
        if (executor.state.value !== "ready") {
            return;
        }
        while (activeCount.value < settings.value.concurrency) {
            const task = tasks.value.find(candidate => candidate.state === "queued");
            if (!task) {
                break;
            }
            activeCount.value += 1;
            void runTask(task).finally(() => {
                activeCount.value = Math.max(0, activeCount.value - 1);
                pumpQueue();
            });
        }
    }

    async function runTask(task: ResolveTask) {
        const cached = task.sourceKey
                ? getCached(task.sourceKey)
                : undefined;
        if (cached) {
            task.result = cached;
            task.state = "ready";
            task.completedAt = Date.now();
            return;
        }

        const controller = new AbortController();
        controllers.set(task.id, controller);
        task.startedAt = Date.now();
        task.error = undefined;

        try {
            const resolved = await executor.resolve(
                    task.id,
                    task.input,
                    controller.signal,
                    phase => {
                        if (!controller.signal.aborted) {
                            task.state = phase;
                        }
                    }
            );

            if (controller.signal.aborted) {
                throw new DOMException("Aborted", "AbortError");
            }

            task.sourceKey = resolved.sourceKey;
            task.normalizedInput = resolved.normalizedInput;
            task.result = resolved.result;
            task.state = "ready";
            task.completedAt = Date.now();
            setCached(resolved.sourceKey, resolved.result);

            if (import.meta.dev) {
                console.debug("task completed", {
                    taskId: task.id,
                    resources: resolved.result.resources.length,
                    durationMs: task.startedAt
                            ? Date.now() - task.startedAt
                            : undefined
                });
            }
        } catch (error) {
            if (isAbort(error) || controller.signal.aborted) {
                if (task.state !== "cancelled") {
                    task.state = "cancelled";
                }
                task.completedAt ||= Date.now();
                return;
            }

            const code = executor.errorCode(error);
            task.error = { code } satisfies TaskError;
            task.state = "failed";
            task.completedAt = Date.now();

            if (import.meta.dev) {
                console.debug("task failed", { taskId: task.id, code });
            }
        } finally {
            controllers.delete(task.id);
        }
    }

    if (import.meta.client && !settingsWatcherInstalled) {
        settingsWatcherInstalled = true;
        watch(() => settings.value.concurrency, () => pumpQueue());
        watch(() => executor.state.value, state => {
            if (state === "ready") {
                pumpQueue();
            }
        });
    }

    return {
        tasks,
        activeCount: readonly(activeCount),
        queuedCount,
        completedCount,
        totalCount,
        add,
        addMany,
        retry,
        cancel,
        remove,
        clearCompleted,
        clearAll,
        canRecover,
        openResponse,
        completeFromResponse,
        select,
        pumpQueue
    };
}

function isAbort(error: unknown) {
    return error instanceof DOMException && error.name === "AbortError";
}
