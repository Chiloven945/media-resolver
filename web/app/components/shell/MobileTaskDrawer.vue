<script lang="ts" setup>
const open = defineModel<boolean>("open", { default: false });
</script>

<template>
  <UDrawer
      v-model:open="open"
      :ui="{ content: 'w-[min(92vw,23.75rem)] max-w-none', body: 'p-0 sm:p-0' }"
      description="Manage active and completed tasks."
      direction="left"
      title="Tasks"
  >
    <template #body>
      <TaskSidebar
          :show-title="false"
          @selected="open = false"
          @task-added="open = false"
      />
    </template>
  </UDrawer>
</template>
