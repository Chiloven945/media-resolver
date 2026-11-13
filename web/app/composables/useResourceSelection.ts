export function useResourceSelection() {
    const selecting = useState<boolean>("resources:selecting", () => false);
    const selectedIds = useState<string[]>("resources:selected", () => []);

    const isSelected = (id: string) => selectedIds.value.includes(id);

    const toggle = (id: string) => {
        if (isSelected(id)) {
            selectedIds.value = selectedIds.value.filter(value => value !== id);
            return;
        }
        selectedIds.value = [...selectedIds.value, id];
    };

    const select = (ids: string[]) => {
        selectedIds.value = [...ids];
    };

    const clear = () => {
        selectedIds.value = [];
    };

    const start = () => {
        selecting.value = true;
    };

    const stop = () => {
        selecting.value = false;
        clear();
    };

    const reset = () => {
        stop();
    };

    return {
        selecting,
        selectedIds,
        isSelected,
        toggle,
        select,
        clear,
        start,
        stop,
        reset
    };
}
