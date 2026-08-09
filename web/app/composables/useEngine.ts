import type {
    EngineErrorShape,
    EngineRuntime,
    EngineState,
    PreparedInput,
    ResourceBundle
} from "~/types/engine";

const runtime = shallowRef<EngineRuntime | null>(null);

export function installEngineRuntime(value: EngineRuntime | null) {
    runtime.value = value;
}

export function useEngine() {
    const state = useState<EngineState>("engine:state", () => "idle");

    const prepare = (input: string): PreparedInput => {
        if (state.value !== "ready" || !runtime.value) {
            throw { code: "internal" };
        }
        return runtime.value.prepare(input);
    };

    const complete = (input: string, status: number, body: Uint8Array): ResourceBundle => {
        if (state.value !== "ready" || !runtime.value) {
            throw { code: "internal" };
        }
        return runtime.value.complete(input, status, body);
    };

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
        prepare,
        complete,
        normalizeError
    };
}
