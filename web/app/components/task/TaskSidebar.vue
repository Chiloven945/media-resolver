<script lang="ts" setup>
import type { BatchGroup } from "~/composables/useBatchDownload";
import { batchEntriesForTask } from "~/composables/useBatchDownload";
import BatchDownloadDialog from "~/components/task/BatchDownloadDialog.vue";

const props = withDefaults(defineProps<{ showTitle?: boolean }>(), {
  showTitle: true
});
const emit = defineEmits<{ selected: []; taskAdded: [] }>();
const clearAllOpen = ref(false);
const batchOpen = ref(false);
const batchArchiveName = ref("");
const query = ref("");
const filter = ref<"all" | "active" | "ready" | "failed">("all");
const queue = useTaskQueue();

const filters = [
  { value: "all", label: "All" },
  { value: "active", label: "Active" },
  { value: "ready", label: "Ready" },
  { value: "failed", label: "Failed" }
] as const;

const filteredTasks = computed(() => {
  const search = query.value.trim().toLowerCase();
  return [...queue.tasks.value].reverse().filter(task => {
    if (filter.value === "active"
        && !["queued", "connecting", "processing"].includes(task.state)) {
      return false;
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
</script>

<template>
  <div class="flex h-full min-h-0 flex-col bg-elevated/30">
    <div v-if="props.showTitle"
         class="flex h-12 shrink-0 items-center justify-between border-b border-default px-4">
      <h2 class="text-sm font-semibold text-highlighted">Tasks</h2>
      <span class="text-xs tabular-nums text-muted">{{ queue.tasks.value.length }}</span>
    </div>

    <div class="flex min-h-0 flex-1 flex-col gap-3 p-3 sm:p-4">
      <TaskComposer @task-added="emit('taskAdded')"/>

      <div v-if="queue.totalCount.value" class="space-y-2 border-b border-default pb-3">
        <div class="flex items-center justify-between text-xs text-muted">
          <span>{{ queue.activeCount.value }} active · {{ queue.queuedCount.value }} queued</span>
          <span>{{ queue.completedCount.value }} / {{ queue.totalCount.value }} complete</span>
        </div>
        <UProgress
            :max="Math.max(queue.totalCount.value, 1)"
            :model-value="progressValue"
            size="xs"
        />
        <UButton
            :disabled="readyTasks.length === 0"
            :label="`Batch download (${readyTasks.length})`"
            block
            color="neutral"
            icon="i-lucide-download"
            size="sm"
            variant="soft"
            @click="openBatch"
        />
      </div>

      <div v-if="queue.totalCount.value" class="space-y-2">
        <UInput
            v-model="query"
            aria-label="Search tasks"
            class="w-full"
            icon="i-lucide-search"
            placeholder="Search by post ID"
            size="sm"
        />
        <div class="flex gap-1 overflow-x-auto">
          <button
              v-for="item in filters"
              :key="item.value"
              :class="filter === item.value
                ? 'border-default bg-default text-highlighted shadow-sm'
                : 'border-transparent text-muted hover:text-highlighted'"
              class="shrink-0 rounded-full border px-2.5 py-1 text-[11px] font-medium transition-colors"
              type="button"
              @click="filter = item.value"
          >
            {{ item.label }}
          </button>
        </div>
      </div>

      <TaskList
          :tasks="filteredTasks"
          :empty-description="queue.tasks.value.length
            ? 'Try a different filter or search term.'
            : 'Add a supported link above to start.'"
          :empty-title="queue.tasks.value.length ? 'No matching tasks' : 'No tasks yet'"
          class="min-h-0 flex-1"
          @selected="emit('selected')"
      />

      <div v-if="queue.tasks.value.length" class="flex items-center border-t border-default pt-2">
        <UButton
            :disabled="queue.completedCount.value === 0"
            class="flex-1 justify-start"
            color="neutral"
            icon="i-lucide-list-x"
            label="Clear completed"
            size="sm"
            variant="ghost"
            @click="queue.clearCompleted"
        />
        <UButton
            aria-label="Clear all tasks"
            color="neutral"
            icon="i-lucide-trash-2"
            size="sm"
            variant="ghost"
            @click="clearAllOpen = true"
        />
      </div>
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
