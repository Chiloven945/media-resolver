<script lang="ts" setup>
import AppHeader from "~/components/shell/AppHeader.vue";
import MobileTaskDrawer from "~/components/shell/MobileTaskDrawer.vue";

const mobileTasksOpen = ref(false);
const { selectedTask } = useTaskSelection();
</script>

<template>
  <div class="app-shell flex flex-col bg-default">
    <AppHeader @open-tasks="mobileTasksOpen = true"/>

    <UDashboardGroup
        :ui="{ base: 'relative inset-auto min-h-0 flex-1' }"
        class="workspace-grid min-h-0 flex-1"
    >
      <UDashboardSidebar
          id="tasks"
          :default-size="22"
          :max-size="24"
          :min-size="18"
          :toggle="false"
          :ui="{ root: 'border-r border-default bg-elevated/30', body: 'p-0 sm:p-0' }"
          class="desktop-task-panel"
          resizable
      >
        <TaskSidebar/>
      </UDashboardSidebar>

      <UDashboardPanel
          id="workspace"
          :ui="{ body: 'min-h-0 flex-1 overflow-hidden p-0 sm:p-0' }"
          class="min-h-0 min-w-0 overflow-hidden bg-default"
      >
        <template #body>
          <ResultView :task="selectedTask"/>
        </template>
      </UDashboardPanel>
    </UDashboardGroup>
  </div>

  <MobileTaskDrawer v-model:open="mobileTasksOpen"/>
</template>
