import { installEngineRuntime } from "~/composables/useEngine";

export default defineNuxtPlugin(() => {
    const state = useState<"idle" | "loading" | "ready" | "error">("engine:state", () => "idle");
    const config = useRuntimeConfig();

    if (state.value !== "idle") {
        return;
    }

    state.value = "loading";

    void (
            async () => {
                try {
                    const modulePath = `${config.app.baseURL}wasm/engine.js`;
                    const moduleVersion = import.meta.dev
                            ? `${Date.now()}`
                            : String(config.public.buildHash || "development");
                    const moduleUrl = `${modulePath}?v=${encodeURIComponent(moduleVersion)}`;
                    const module = await import(/* @vite-ignore */ moduleUrl);
                    await module.default();
                    installEngineRuntime({
                        start: module.start,
                        respond: module.respond,
                        transportFailed: module.transport_failed
                    });
                    state.value = "ready";
                } catch (error) {
                    installEngineRuntime(null);
                    state.value = "error";
                    if (import.meta.dev) {
                        console.debug("engine initialization failed", {
                            name: error instanceof Error
                                    ? error.name
                                    : "unknown"
                        });
                    }
                }
            }
    )();
});
