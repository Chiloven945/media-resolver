<script lang="ts" setup>
const props = withDefaults(defineProps<{ showTitle?: boolean }>(), {
  showTitle: true
});
const emit = defineEmits<{ selected: []; taskAdded: [] }>();
const clearAllOpen = ref(false);
const queue = useTaskQueue();

const progressValue = computed(() => queue.totalCount.value
    ? queue.completedCount.value
    : 0);
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
      </div>

      <TaskList class="min-h-0 flex-1" @selected="emit('selected')"/>

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
