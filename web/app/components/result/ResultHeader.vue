<script lang="ts" setup>
import type { DropdownMenuItem } from "@nuxt/ui";
import type { ResolveTask } from "~/types/task";

const props = defineProps<{
  task: ResolveTask;
  resourceCount: number;
  completedLabel: string;
}>();
const queue = useTaskQueue();
const toast = useToast();

const menuItems = computed<DropdownMenuItem[][]>(() => {
  const items: DropdownMenuItem[] = [];
  if (["connecting", "processing"].includes(props.task.state)) {
    items.push({
      label: "Cancel",
      icon: "i-lucide-circle-stop",
      onSelect: () => queue.cancel(props.task.id)
    });
  } else {
    items.push({
      label: props.task.state === "ready" ? "Run again" : "Retry",
      icon: "i-lucide-refresh-cw",
      onSelect: () => {
        queue.retry(props.task.id);
        toast.add({ title: "Retry started", color: "neutral" });
      }
    });
  }
  items.push({
    label: "Copy link",
    icon: "i-lucide-link",
    onSelect: async () => {
      try {
        await navigator.clipboard.writeText(props.task.normalizedInput || props.task.input);
        toast.add({ title: "Link copied", color: "success" });
      } catch {
        toast.add({ title: "Unable to copy link", color: "error" });
      }
    }
  });
  items.push({
    label: "Remove task",
    icon: "i-lucide-trash-2",
    color: "error",
    onSelect: () => queue.remove(props.task.id)
  });
  return [items];
});

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
  <header
      class="flex min-h-19 shrink-0 items-center justify-between gap-4 border-b border-default bg-default px-4 py-3 sm:px-5"
      data-testid="result-header">
    <div class="min-w-0">
      <div class="flex items-center gap-2">
        <h1 class="truncate font-mono text-base font-semibold text-highlighted sm:text-lg">
          {{ task.sourceKey || `Task ${task.sequence}` }}
        </h1>
        <TaskStatus :state="task.state"/>
      </div>
      <p class="mt-1 truncate text-sm text-muted">
        <span class="font-medium text-muted">Task {{ task.sequence }}</span>
        <span class="px-1.5 text-muted">·</span>
        {{ summary }}
      </p>
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
      <UDropdownMenu :items="menuItems">
        <UButton
            aria-label="Result actions"
            color="neutral"
            icon="i-lucide-ellipsis-vertical"
            size="sm"
            variant="ghost"
        />
      </UDropdownMenu>
    </div>
  </header>
</template>
