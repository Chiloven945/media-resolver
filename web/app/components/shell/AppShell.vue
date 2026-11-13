<script lang="ts" setup>
import AppHeader from "~/components/shell/AppHeader.vue";
import MobileTasksView from "~/components/task/MobileTasksView.vue";

const { pane, openResult, openTasks } = useMobileNavigation();
const { selectedTask } = useTaskSelection();

watch(selectedTask, task => {
  if (!task && pane.value === "result") {
    openTasks();
  }
});
</script>

<template>
  <div class="app-shell flex flex-col bg-default">
    <AppHeader/>

    <div class="flex min-h-0 flex-1">
      <aside
          class="desktop-task-panel hidden w-[22.5rem] shrink-0 border-r border-default bg-elevated/20 lg:flex lg:flex-col"
      >
        <TaskSidebar/>
      </aside>

      <main class="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden bg-default">
        <div v-show="pane === 'tasks'" class="h-full min-h-0 lg:hidden">
          <MobileTasksView @selected="openResult"/>
        </div>

        <div
            :class="pane === 'result' ? 'flex' : 'hidden lg:flex'"
            class="h-full min-h-0 flex-col"
        >
          <ResultView :key="`${selectedTask?.id || 'empty'}-${pane}`" :task="selectedTask"/>
        </div>
      </main>
    </div>
  </div>
</template>
