<script lang="ts" setup>
import type { BatchGroup } from "~/composables/useBatchDownload";
import { batchEntriesForTask } from "~/composables/useBatchDownload";
import TaskCard from "~/components/task/TaskCard.vue";
import BatchDownloadDialog from "~/components/task/BatchDownloadDialog.vue";

const emit = defineEmits<{ selected: [] }>();
const queue = useTaskQueue();
const { settings } = useAppSettings();
const clearAllOpen = ref(false);
const batchOpen = ref(false);
const batchArchiveName = ref("");
const query = ref("");
const filter = ref<"all" | "active" | "ready" | "failed">("all");

const filters = [
  { value: "all", label: "All" },
  { value: "active", label: "Active" },
  { value: "ready", label: "Ready" },
  { value: "failed", label: "Failed" }
] as const;

const filteredTasks = computed(() => {
  const search = query.value.trim().toLowerCase();
  return [...queue.tasks.value].reverse().filter(task => {
    if (filter.value === "active") {
      if (!["queued", "connecting", "processing"].includes(task.state)) {
        return false;
      }
    }
    if (filter.value === "ready" && task.state !== "ready") {
      return false;
    }
    if (filter.value === "failed" && !["failed", "cancelled"].includes(task.state)) {
      return false;
    }
    if (!search) {
      return true;
    }
    return [task.sourceKey, task.input, task.normalizedInput]
        .filter(Boolean)
        .some(value => value!.toLowerCase().includes(search));
  });
});

const progressValue = computed(() => queue.totalCount.value
    ? queue.completedCount.value
    : 0);
const readyTasks = computed(() => queue.tasks.value.filter(task =>
    task.state === "ready" && (task.result?.resources.length || 0) > 0
));
const batchGroups = computed<BatchGroup[]>(() => readyTasks.value.map(task => ({
  id: task.id,
  title: task.sourceKey || `Task ${task.sequence}`,
  subtitle: `${task.result!.resources.length} resources · Task ${task.sequence}`,
  entries: batchEntriesForTask(task)
})));

const openBatch = () => {
  batchArchiveName.value = `media-resolver-${new Date().toISOString()
      .replace(/[:T]/g, "-")
      .slice(0, 16)}.zip`;
  batchOpen.value = true;
};

const taskAdded = (count: number) => {
  if (count === 1 && settings.value.autoSelect) {
    emit("selected");
  }
};
</script>

<template>
  <div class="flex h-full min-h-0 flex-col bg-elevated/20" data-testid="mobile-tasks-view">
    <div class="shrink-0 space-y-3 border-b border-default bg-elevated/20 p-3">
      <TaskComposer @task-added="taskAdded"/>

      <div v-if="queue.totalCount.value" class="space-y-2">
        <div class="flex items-center justify-between text-xs text-muted">
          <span>{{ queue.activeCount.value }} active · {{ queue.queuedCount.value }} queued</span>
          <span>{{ queue.completedCount.value }} / {{ queue.totalCount.value }} complete</span>
        </div>
        <UProgress
            :max="Math.max(queue.totalCount.value, 1)"
            :model-value="progressValue"
            size="xs"
        />
      </div>

      <div class="flex items-center gap-2">
        <UInput
            v-model="query"
            aria-label="Search tasks"
            class="min-w-0 flex-1"
            icon="i-lucide-search"
            placeholder="Search by post ID"
            size="sm"
        />
        <UDropdownMenu
            :items="[[
              { label: 'Clear completed', icon: 'i-lucide-list-x', onSelect: queue.clearCompleted },
              { label: 'Clear all tasks', icon: 'i-lucide-trash-2', color: 'error', onSelect: () => clearAllOpen = true }
            ]]"
        >
          <UButton
              aria-label="Task list actions"
              color="neutral"
              icon="i-lucide-ellipsis-vertical"
              size="sm"
              variant="outline"
          />
        </UDropdownMenu>
      </div>

      <div class="flex gap-1 overflow-x-auto pb-0.5">
        <button
            v-for="item in filters"
            :key="item.value"
            :class="filter === item.value
              ? 'bg-default text-highlighted shadow-sm'
              : 'text-muted hover:text-highlighted'"
            class="shrink-0 rounded-full border border-transparent px-3 py-1 text-xs font-medium transition-colors"
            type="button"
            @click="filter = item.value"
        >
          {{ item.label }}
        </button>
      </div>
    </div>

    <div class="min-h-0 flex-1 overflow-y-auto overscroll-contain p-3">
      <div v-if="!filteredTasks.length" class="px-4 py-12 text-center">
        <UIcon class="mx-auto size-7 text-muted" name="i-lucide-list-todo"/>
        <div class="mt-3 text-sm font-medium text-highlighted">
          {{ queue.tasks.value.length ? "No matching tasks" : "No tasks yet" }}
        </div>
        <p class="mx-auto mt-1 max-w-xs text-xs leading-5 text-muted">
          {{ queue.tasks.value.length
            ? "Try a different filter or search term."
            : "Paste a supported link above to get started." }}
        </p>
      </div>

      <div v-else class="space-y-2">
        <TaskCard
            v-for="task in filteredTasks"
            :key="task.id"
            :task="task"
            @selected="emit('selected')"
        />
      </div>
    </div>

    <div
        class="shrink-0 border-t border-default bg-default/95 px-3 py-3 pb-[calc(0.75rem+env(safe-area-inset-bottom))] backdrop-blur"
    >
      <UButton
          :disabled="readyTasks.length === 0"
          :label="`Batch download (${readyTasks.length})`"
          block
          class="min-h-11"
          color="neutral"
          icon="i-lucide-download"
          size="md"
          variant="soft"
          @click="openBatch"
      />
    </div>
  </div>

  <BatchDownloadDialog
      v-model:open="batchOpen"
      :archive-name="batchArchiveName"
      :groups="batchGroups"
      description="Select completed tasks and choose how to save their resources."
  />

  <UModal
      v-model:open="clearAllOpen"
      description="Active tasks will be cancelled and all task state will be removed."
      title="Clear all tasks?"
  >
    <template #footer>
      <div class="flex w-full justify-end gap-2">
        <UButton color="neutral" label="Cancel" variant="ghost" @click="clearAllOpen = false"/>
        <UButton
            color="error"
            icon="i-lucide-trash-2"
            label="Clear all"
            @click="queue.clearAll(); clearAllOpen = false"
        />
      </div>
    </template>
  </UModal>
</template>
