<script lang="ts" setup>
import type { DropdownMenuItem } from "@nuxt/ui";
import type { ResolveTask } from "~/types/task";

const props = defineProps<{ task: ResolveTask }>();
const emit = defineEmits<{ selected: [] }>();
const queue = useTaskQueue();
const { selectedId, select } = useTaskSelection();
const toast = useToast();

const isSelected = computed(() => selectedId.value === props.task.id);
const summary = computed(() => {
  const count = props.task.result?.resources.length;
  if (props.task.state === "ready" && typeof count === "number") {
    return `${count} ${count === 1
        ? "resource"
        : "resources"}`;
  }
  if (props.task.state === "failed") {
    return "Needs attention";
  }
  if (props.task.state === "cancelled") {
    return "Cancelled";
  }
  return "Task in progress";
});

const selectTask = () => {
  select(props.task.id);
  emit("selected");
};

const menuItems = computed(() => {
  const actions: DropdownMenuItem[] = [];
  if (props.task.state === "connecting" || props.task.state === "processing") {
    actions.push({
      label: "Cancel",
      icon: "i-lucide-circle-stop",
      onSelect: () => queue.cancel(props.task.id)
    });
  } else if (props.task.state === "ready") {
    actions.push({
      label: "Run again",
      icon: "i-lucide-refresh-cw",
      onSelect: () => {
        queue.retry(props.task.id);
        toast.add({ title: "Retry started", color: "neutral" });
      }
    });
  } else if (props.task.state === "failed" || props.task.state === "cancelled") {
    actions.push({
      label: "Retry",
      icon: "i-lucide-refresh-cw",
      onSelect: () => {
        queue.retry(props.task.id);
        toast.add({ title: "Retry started", color: "neutral" });
      }
    });
  }

  if (props.task.state !== "connecting" && props.task.state !== "processing") {
    actions.push({
      label: "Remove",
      icon: "i-lucide-trash-2",
      color: "error",
      onSelect: () => {
        queue.remove(props.task.id);
        toast.add({ title: "Task removed", color: "neutral" });
      }
    });
  }

  return [actions];
});
</script>

<template>
  <div
      :class="isSelected ? 'border-primary bg-elevated' : 'border-transparent hover:bg-elevated/70'"
      :data-task-sequence="task.sequence"
      class="flex items-stretch border-l-2 transition-colors"
  >
    <UButton
        class="min-w-0 flex-1 justify-start px-3 py-2.5"
        color="neutral"
        variant="ghost"
        @click="selectTask"
    >
      <div class="min-w-0 flex-1 text-left">
        <div class="flex items-center justify-between gap-2">
          <span class="truncate text-sm font-medium text-highlighted">Task {{
              task.sequence
            }}</span>
          <TaskStatus :state="task.state"/>
        </div>
        <div class="mt-1 truncate text-xs text-muted">{{ summary }}</div>
      </div>
    </UButton>

    <UDropdownMenu :items="menuItems">
      <UButton
          aria-label="Task actions"
          color="neutral"
          icon="i-lucide-ellipsis-vertical"
          size="sm"
          variant="ghost"
      />
    </UDropdownMenu>
  </div>
</template>
