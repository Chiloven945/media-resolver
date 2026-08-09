<script setup lang="ts">
const emit = defineEmits<{ selected: [] }>();
const queue = useTaskQueue();
</script>

<template>
  <div class="min-h-0 overflow-y-auto">
    <TaskEmpty v-if="!queue.tasks.value.length"/>
    <div v-else class="space-y-1.5">
      <TaskListItem
          v-for="task in [...queue.tasks.value].reverse()"
          :key="task.id"
          :task="task"
          @selected="emit('selected')"
      />
    </div>
  </div>
</template>
