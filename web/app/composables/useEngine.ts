import type { EngineErrorShape, EngineRuntime, EngineState } from "~/types/engine";
import type {
    ResolutionOptions,
    ResolutionSession,
    ResolutionStep,
    TransportFailureKind
} from "~/types/resolution";

const runtime = shallowRef<EngineRuntime | null>(null);

export function installEngineRuntime(value: EngineRuntime | null) {
    runtime.value = value;
}

export function useEngine() {
    const state = useState<EngineState>("engine:state", () => "idle");

    const requireRuntime = (): EngineRuntime => {
        if (state.value !== "ready" || !runtime.value) {
            throw { code: "internal" };
        }
        return runtime.value;
    };

    const start = (input: string, options: ResolutionOptions): ResolutionStep =>
            requireRuntime().start(input, options);

    const respond = (
            session: ResolutionSession,
            status: number,
            body: Uint8Array
    ): ResolutionStep => requireRuntime().respond(session, status, body);

    const transportFailed = (
            session: ResolutionSession,
            kind: TransportFailureKind
    ): ResolutionStep => requireRuntime().transportFailed(session, kind);

    const normalizeError = (error: unknown): EngineErrorShape => {
        if (error && typeof error === "object") {
            const shape = error as EngineErrorShape;
            return {
                code: typeof shape.code === "string"
                        ? shape.code
                        : "internal",
                message: typeof shape.message === "string"
                        ? shape.message
                        : undefined
            };
        }
        return { code: "internal" };
    };

    return {
        state: readonly(state),
        runtime: readonly(runtime),
        start,
        respond,
        transportFailed,
        normalizeError
    };
}
