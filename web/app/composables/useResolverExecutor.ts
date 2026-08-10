import { ref } from "vue";
import type { ResourceBundle } from "~/types/engine";
import type {
    PreparedRequest,
    ResolutionOptions,
    ResolutionSession,
    TransportFailureKind
} from "~/types/resolution";
import { executeRequest, type TransportResult } from "~/utils/transport";
import { resolverEndpoint } from "~/utils/resolver-endpoint";

interface InspectionResult {
    sourceKey: string;
    normalizedInput: string;
}

interface ResolveResult extends InspectionResult {
    result: ResourceBundle;
}

interface RecoveryContext {
    session: ResolutionSession;
    request: PreparedRequest;
}

interface RouteHealth {
    unavailableUntil: number;
    reason: "access_blocked" | "network" | "rate_limited";
}

export type ResolverPhase = "connecting" | "processing";

const routeHealth = new Map<string, RouteHealth>();
const recoveryContexts = new Map<string, RecoveryContext>();
const recoveryRevision = ref(0);
const EMPTY_BODY = new Uint8Array();
const ACCESS_BLOCKED_TTL_MS = 10 * 60 * 1000;
const NETWORK_TTL_MS = 30 * 1000;
const RATE_LIMIT_TTL_MS = 30 * 1000;

function rememberRecovery(taskId: string, context: RecoveryContext) {
    recoveryContexts.set(taskId, context);
    recoveryRevision.value += 1;
}

function forgetRecovery(taskId: string) {
    if (recoveryContexts.delete(taskId)) {
        recoveryRevision.value += 1;
    }
}

export function useResolverExecutor() {
    const engine = useEngine();
    const config = useRuntimeConfig();

    const resolutionOptions = (): ResolutionOptions => {
        return {
            profile: "browser",
            gatewayEndpoint: resolverEndpoint(config.public.resolverEndpoint)
        };
    };

    const inspect = (input: string): InspectionResult => {
        const step = engine.start(input, resolutionOptions());
        if (step.kind === "request") {
            return {
                sourceKey: step.sourceKey,
                normalizedInput: step.normalizedInput
            };
        }
        if (step.kind === "resolved") {
            return {
                sourceKey: step.result.sourceKey,
                normalizedInput: input.trim()
            };
        }
        throw step.error;
    };

    const resolve = async (
            taskId: string,
            input: string,
            signal: AbortSignal,
            onPhase?: (phase: ResolverPhase) => void
    ): Promise<ResolveResult> => {
        let step = engine.start(input, resolutionOptions());
        let sourceKey = "";
        let normalizedInput = input.trim();

        while (true) {
            if (signal.aborted) {
                throw new DOMException("Aborted", "AbortError");
            }

            if (step.kind === "resolved") {
                forgetRecovery(taskId);
                return {
                    sourceKey: step.result.sourceKey || sourceKey,
                    normalizedInput,
                    result: step.result
                };
            }
            if (step.kind === "failed") {
                if (step.error.code
                        === "remote_restricted"
                        || step.error.code
                        === "remote_not_found") {
                    forgetRecovery(taskId);
                }
                throw step.error;
            }

            sourceKey ||= step.sourceKey;
            normalizedInput = step.normalizedInput || normalizedInput;
            const health = getRouteHealth(step.request.routeKey);
            if (health) {
                if (health.reason === "access_blocked") {
                    rememberRecovery(taskId, {
                        session: step.session,
                        request: step.request
                    });
                    step = engine.transportFailed(step.session, "access_blocked");
                } else if (health.reason === "rate_limited") {
                    step = engine.respond(step.session, 429, EMPTY_BODY);
                } else {
                    step = engine.transportFailed(step.session, "network");
                }
                continue;
            }

            onPhase?.("connecting");
            const transport = await executeWithRetry(step.request, signal);
            if (transport.kind === "response") {
                if (transport.status === 429) {
                    markRouteUnavailable(
                            step.request.routeKey,
                            "rate_limited",
                            transport.retryAfterMs || RATE_LIMIT_TTL_MS
                    );
                }
                onPhase?.("processing");
                step = engine.respond(step.session, transport.status, transport.body);
                continue;
            }

            const failure = transportFailureKind(transport);
            if (transport.kind === "access_blocked") {
                rememberRecovery(taskId, {
                    session: step.session,
                    request: step.request
                });
                markRouteUnavailable(
                        step.request.routeKey,
                        "access_blocked",
                        ACCESS_BLOCKED_TTL_MS
                );
            } else {
                markRouteUnavailable(step.request.routeKey, "network", NETWORK_TTL_MS);
            }
            step = engine.transportFailed(step.session, failure);
        }
    };

    const continueFromRecovery = (taskId: string, body: string): ResourceBundle => {
        const context = recoveryContexts.get(taskId);
        if (!context) {
            throw { code: "remote_unavailable" };
        }
        const bytes = new TextEncoder().encode(body.trim());
        if (!bytes.length) {
            throw { code: "invalid_response" };
        }
        const step = engine.respond(context.session, 200, bytes);
        if (step.kind === "resolved") {
            forgetRecovery(taskId);
            return step.result;
        }
        if (step.kind === "failed") {
            throw step.error;
        }
        throw { code: "remote_unavailable" };
    };

    const hasRecovery = (taskId: string): boolean => {
        void recoveryRevision.value;
        return recoveryContexts.has(taskId);
    };

    const openRecoveryResponse = (taskId: string): boolean => {
        const context = recoveryContexts.get(taskId);
        if (!context || !import.meta.client) {
            return false;
        }
        try {
            const url = new URL(context.request.url);
            if (url.protocol !== "https:") {
                return false;
            }
            window.open(url.toString(), "_blank", "noopener,noreferrer");
            return true;
        } catch {
            return false;
        }
    };

    const clearRecovery = (taskId: string) => {
        forgetRecovery(taskId);
    };

    const errorCode = (error: unknown): string =>
            engine.normalizeError(error).code || "internal";

    return {
        state: engine.state,
        inspect,
        resolve,
        continueFromRecovery,
        hasRecovery,
        openRecoveryResponse,
        clearRecovery,
        errorCode
    };
}

async function executeWithRetry(
        request: PreparedRequest,
        signal: AbortSignal
): Promise<TransportResult> {
    const policy = request.retryPolicy;
    for (let attempt = 0; attempt <= policy.maxRetries; attempt += 1) {
        if (signal.aborted) {
            throw new DOMException("Aborted", "AbortError");
        }
        const result = await executeRequest(request, signal);
        const retryableResponse = result.kind === "response"
                && policy.retryStatuses.includes(result.status);
        const retryableTransport = result.kind === "network_error" || result.kind === "timeout";

        if ((
                !retryableResponse && !retryableTransport
        ) || attempt >= policy.maxRetries) {
            return result;
        }

        const retryAfter = result.kind === "response"
                ? result.retryAfterMs
                : undefined;
        await retryDelay(request, attempt, retryAfter, signal);
    }
    return { kind: "network_error" };
}

function transportFailureKind(result: Exclude<TransportResult, {
    kind: "response"
}>): TransportFailureKind {
    if (result.kind === "access_blocked") {
        return "access_blocked";
    }
    if (result.kind === "timeout") {
        return "timeout";
    }
    return "network";
}

function getRouteHealth(routeKey: string): RouteHealth | undefined {
    const health = routeHealth.get(routeKey);
    if (!health) {
        return undefined;
    }
    if (health.unavailableUntil <= Date.now()) {
        routeHealth.delete(routeKey);
        return undefined;
    }
    return health;
}

function markRouteUnavailable(
        routeKey: string,
        reason: RouteHealth["reason"],
        ttlMs: number
) {
    routeHealth.set(routeKey, {
        reason,
        unavailableUntil: Date.now() + Math.max(1_000, ttlMs)
    });
}

async function retryDelay(
        request: PreparedRequest,
        attempt: number,
        retryAfterMs: number | undefined,
        signal: AbortSignal
) {
    const configured = request.retryPolicy.delaysMs[attempt]
            ?? request.retryPolicy.delaysMs.at(-1)
            ?? 500;
    const base = retryAfterMs ?? configured;
    const jitter = retryAfterMs
            ? 0
            : Math.floor(Math.random() * 180);
    await new Promise<void>((resolve, reject) => {
        const onAbort = () => {
            window.clearTimeout(timer);
            reject(new DOMException("Aborted", "AbortError"));
        };
        const timer = window.setTimeout(() => {
            signal.removeEventListener("abort", onAbort);
            resolve();
        }, base + jitter);
        signal.addEventListener("abort", onAbort, { once: true });
    });
}
