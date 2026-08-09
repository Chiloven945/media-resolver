export function useTaskSelection() {
    const tasks = useState<import("~/types/task").ResolveTask[]>("tasks:list", () => []);
    const selectedId = useState<string | null>("tasks:selected", () => null);

    const selectedTask = computed(() =>
            tasks.value.find(task => task.id === selectedId.value) || null
    );

    const select = (id: string | null) => {
        selectedId.value = id;
    };

    return {
        selectedId,
        selectedTask,
        select
    };
}
