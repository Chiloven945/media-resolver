<script setup lang="ts">
import MobileTaskDrawer from "~/components/shell/MobileTaskDrawer.vue";
import AppNavbar from "~/components/shell/AppNavbar.vue";

const mobileTasksOpen = ref(false);
const clearAllOpen = ref(false);
const queue = useTaskQueue();
const { selectedTask } = useTaskSelection();

const progressValue = computed(() => queue.totalCount.value
    ? queue.completedCount.value
    : 0);
</script>

<template>
  <UDashboardGroup class="workspace-grid">
    <UDashboardPanel
        id="tasks"
        class="desktop-task-panel"
        :default-size="28"
        :min-size="22"
        :max-size="36"
        resizable
    >
      <UDashboardNavbar title="Tasks">
        <template #right>
          <UBadge color="neutral" variant="subtle">
            {{ queue.tasks.value.length }}
          </UBadge>
        </template>
      </UDashboardNavbar>

      <div class="flex min-h-0 flex-1 flex-col gap-4 p-4">
        <TaskComposer/>

        <div v-if="queue.totalCount.value" class="space-y-2">
          <div class="flex items-center justify-between text-xs text-muted">
            <span>{{ queue.activeCount.value }} active · {{ queue.queuedCount.value }} queued</span>
            <span>{{ queue.completedCount.value }} / {{ queue.totalCount.value }} completed</span>
          </div>
          <UProgress
              :model-value="progressValue"
              :max="Math.max(queue.totalCount.value, 1)"
              size="xs"
          />
        </div>

        <TaskList class="min-h-0 flex-1"/>

        <div v-if="queue.tasks.value.length" class="flex gap-2 border-t border-default pt-3">
          <UButton
              label="Clear completed"
              icon="i-lucide-list-x"
              color="neutral"
              variant="ghost"
              size="sm"
              class="flex-1"
              :disabled="queue.completedCount.value === 0"
              @click="queue.clearCompleted"
          />
          <UButton
              icon="i-lucide-trash-2"
              color="neutral"
              variant="ghost"
              size="sm"
              aria-label="Clear all tasks"
              @click="clearAllOpen = true"
          />
        </div>
      </div>
    </UDashboardPanel>

    <UDashboardPanel id="workspace" class="min-w-0">
      <AppNavbar @open-tasks="mobileTasksOpen = true"/>
      <div class="min-h-0 flex-1 overflow-y-auto">
        <ResultView
            :task="selectedTask"
            @new-task="mobileTasksOpen = true"
        />
      </div>
    </UDashboardPanel>
  </UDashboardGroup>

  <MobileTaskDrawer v-model:open="mobileTasksOpen"/>

  <UModal v-model:open="clearAllOpen" title="Clear all tasks?"
          description="Active tasks will be cancelled and all task state will be removed.">
    <template #footer>
      <div class="flex w-full justify-end gap-2">
        <UButton label="Cancel" color="neutral" variant="ghost" @click="clearAllOpen = false"/>
        <UButton
            label="Clear all"
            icon="i-lucide-trash-2"
            color="error"
            @click="queue.clearAll(); clearAllOpen = false"
        />
      </div>
    </template>
  </UModal>
</template>
