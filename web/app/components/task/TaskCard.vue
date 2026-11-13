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
const thumbnails = computed(() => (props.task.result?.resources || [])
    .slice(0, 3)
    .map(resource => resource.previewUrl || resource.preferredUrl)
    .filter(Boolean));

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
      :class="isSelected
        ? 'border-primary bg-elevated shadow-sm'
        : 'border-default bg-default hover:border-primary/30 hover:bg-elevated/60'"
      :data-task-sequence="task.sequence"
      class="group flex items-stretch gap-1 rounded-xl border transition-colors"
      data-testid="task-card"
  >
    <UButton
        class="min-w-0 flex-1 justify-start gap-3 rounded-xl px-2.5 py-2.5"
        color="neutral"
        variant="ghost"
        @click="selectTask"
    >
      <div class="flex shrink-0 -space-x-2">
        <template v-if="thumbnails.length">
          <img
              v-for="(thumbnail, index) in thumbnails"
              :key="`${task.id}-${index}`"
              :src="thumbnail"
              alt=""
              class="size-10 rounded-lg border-2 border-default object-cover shadow-sm"
              loading="lazy"
              referrerpolicy="no-referrer"
          >
        </template>
        <div
            v-else
            class="flex size-10 items-center justify-center rounded-lg border border-dashed border-default bg-elevated text-muted"
        >
          <UIcon :name="task.state === 'failed' ? 'i-lucide-triangle-alert' : 'i-lucide-image'"
                 class="size-4"/>
        </div>
      </div>

      <div class="min-w-0 flex-1 text-left">
        <div class="flex items-center justify-between gap-2">
          <span class="truncate text-sm font-semibold text-highlighted">
            {{ task.sourceKey || `Task ${task.sequence}` }}
          </span>
          <TaskStatus :state="task.state" class="shrink-0"/>
        </div>
        <div class="mt-1 truncate text-xs text-muted">Task {{ task.sequence }} · {{ summary }}</div>
      </div>
    </UButton>

    <UDropdownMenu :items="menuItems">
      <UButton
          aria-label="Task actions"
          class="my-2 mr-1 rounded-lg"
          color="neutral"
          icon="i-lucide-ellipsis-vertical"
          size="sm"
          variant="ghost"
      />
    </UDropdownMenu>
  </div>
</template>
