export interface AppSettings {
    concurrency: number;
    autoSelect: boolean;
    autoPreview: boolean;
}

const STORAGE_KEY = "media-resolver.preferences.v1";
let initialized = false;

export function useAppSettings() {
    const settings = useState<AppSettings>(
            "app:settings",
            () => (
                    {
                        concurrency: 4,
                        autoSelect: true,
                        autoPreview: false
                    }
            )
    );

    if (import.meta.client && !initialized) {
        initialized = true;
        onMounted(() => {
            try {
                const raw = localStorage.getItem(STORAGE_KEY);
                if (!raw) {
                    return;
                }
                const stored = JSON.parse(raw) as Partial<AppSettings>;
                if (typeof stored.concurrency === "number") {
                    settings.value.concurrency =
                            Math.min(8, Math.max(1, Math.round(stored.concurrency)));
                }
                if (typeof stored.autoSelect === "boolean") {
                    settings.value.autoSelect =
                            stored.autoSelect;
                }
                if (typeof stored.autoPreview === "boolean") {
                    settings.value.autoPreview =
                            stored.autoPreview;
                }
            } catch {
                // Ignore invalid local preferences and keep safe defaults.
            }
        });

        watch(settings, (value) => {
            localStorage.setItem(STORAGE_KEY, JSON.stringify({
                concurrency: value.concurrency,
                autoSelect: value.autoSelect,
                autoPreview: value.autoPreview
            }));
        }, { deep: true });
    }

    return { settings };
}
