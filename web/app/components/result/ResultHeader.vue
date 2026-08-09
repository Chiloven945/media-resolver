<script lang="ts" setup>
import type { ResolveTask } from "~/types/task";

const props = defineProps<{
  task: ResolveTask;
  resourceCount: number;
  completedLabel: string;
}>();
const queue = useTaskQueue();

const summary = computed(() => {
  switch (props.task.state) {
    case "ready": {
      const resourceLabel = `${props.resourceCount} ${
          props.resourceCount === 1
              ? "resource"
              : "resources"
      }`;
      return props.completedLabel
          ? `${resourceLabel} · ${props.completedLabel}`
          : resourceLabel;
    }
    case "queued":
      return "Waiting for an available processing slot";
    case "connecting":
      return "Connecting to an available source";
    case "processing":
      return "Processing the response";
    case "cancelled":
      return "Task cancelled";
    default:
      return "Task needs attention";
  }
});
</script>

<template>
  <header class="flex min-h-19 shrink-0 items-center justify-between gap-4 border-b border-default px-4 py-3 sm:px-5"
          data-testid="result-header">
    <div class="min-w-0">
      <div class="flex items-center gap-2">
        <h1 class="truncate text-lg font-semibold text-highlighted">Task {{ task.sequence }}</h1>
        <TaskStatus :state="task.state"/>
      </div>
      <p class="mt-1 truncate text-sm text-muted">{{ summary }}</p>
    </div>

    <div class="flex shrink-0 items-center gap-2">
      <UButton
          v-if="task.state === 'connecting' || task.state === 'processing'"
          color="neutral"
          icon="i-lucide-circle-stop"
          label="Cancel"
          size="sm"
          variant="outline"
          @click="queue.cancel(task.id)"
      />
      <UButton
          v-else-if="task.state === 'cancelled' || task.state === 'failed'"
          icon="i-lucide-refresh-cw"
          label="Retry"
          size="sm"
          @click="queue.retry(task.id)"
      />
    </div>
  </header>
</template>
