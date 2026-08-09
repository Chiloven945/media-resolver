<script setup lang="ts">
const open = defineModel<boolean>("open", { default: false });
const clearAllOpen = ref(false);
const queue = useTaskQueue();
const progressValue = computed(() => queue.totalCount.value
    ? queue.completedCount.value
    : 0);
</script>

<template>
  <UDrawer v-model:open="open"
           direction="left"
           title="Tasks"
           description="Manage active and completed tasks."
           :ui="{ content: 'w-[min(92vw,24rem)] max-w-none' }">
    <template #body>
      <div class="flex h-full min-h-0 flex-col gap-4">
        <TaskComposer @task-added="open = false"/>

        <div v-if="queue.totalCount.value" class="space-y-2">
          <div class="flex items-center justify-between text-xs text-muted">
            <span>{{ queue.activeCount.value }} active · {{ queue.queuedCount.value }} queued</span>
            <span>{{ queue.completedCount.value }} / {{ queue.totalCount.value }}</span>
          </div>
          <UProgress :model-value="progressValue"
                     :max="Math.max(queue.totalCount.value, 1)"
                     size="xs"/>
        </div>

        <TaskList class="min-h-0 flex-1" @selected="open = false"/>

        <div v-if="queue.tasks.value.length" class="flex gap-2 border-t border-default pt-3">
          <UButton
              label="Clear completed"
              color="neutral"
              variant="ghost"
              class="flex-1"
              :disabled="queue.completedCount.value === 0"
              @click="queue.clearCompleted"
          />
          <UButton
              icon="i-lucide-trash-2"
              color="neutral"
              variant="ghost"
              aria-label="Clear all tasks"
              @click="clearAllOpen = true"
          />
        </div>
      </div>
    </template>
  </UDrawer>

  <UModal v-model:open="clearAllOpen" title="Clear all tasks?"
          description="Active tasks will be cancelled and all task state will be removed.">
    <template #footer>
      <div class="flex w-full justify-end gap-2">
        <UButton label="Cancel" color="neutral" variant="ghost" @click="clearAllOpen = false"/>
        <UButton label="Clear all"
                 color="error"
                 @click="queue.clearAll(); clearAllOpen = false; open = false"/>
      </div>
    </template>
  </UModal>
</template>
