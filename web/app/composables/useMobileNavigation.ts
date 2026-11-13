export type MobilePane = "tasks" | "result";

const HISTORY_KEY = "mediaResolverPane";

export function useMobileNavigation() {
    const pane = useState<MobilePane>("mobile:pane", () => "tasks");

    const setPane = (value: MobilePane) => {
        pane.value = value;
    };

    const openTasks = () => {
        setPane("tasks");
        if (import.meta.client && window.history.state?.[HISTORY_KEY] === "result") {
            window.history.back();
        }
    };

    const openResult = () => {
        if (import.meta.client && !window.matchMedia("(min-width: 1024px)").matches) {
            if (pane.value !== "result") {
                window.history.pushState({ [HISTORY_KEY]: "result" }, "");
            }
        }
        setPane("result");
    };

    onMounted(() => {
        if (!import.meta.client) {
            return;
        }

        window.history.replaceState(
            { ...window.history.state, [HISTORY_KEY]: pane.value },
            ""
        );

        const handlePopState = (event: PopStateEvent) => {
            setPane(event.state?.[HISTORY_KEY] === "result"
                ? "result"
                : "tasks");
        };

        window.addEventListener("popstate", handlePopState);
        onBeforeUnmount(() => window.removeEventListener("popstate", handlePopState));
    });

    return {
        pane,
        openTasks,
        openResult,
        isTasks: computed(() => pane.value === "tasks"),
        isResult: computed(() => pane.value === "result")
    };
}
