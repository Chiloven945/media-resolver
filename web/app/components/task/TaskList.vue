<script lang="ts" setup>
import type { ResolveTask } from "~/types/task";
import TaskCard from "~/components/task/TaskCard.vue";

const props = withDefaults(defineProps<{
  tasks?: ResolveTask[];
  emptyTitle?: string;
  emptyDescription?: string;
}>(), {
  emptyTitle: "No tasks yet",
  emptyDescription: "Add a supported link above to start."
});
const emit = defineEmits<{ selected: [] }>();
const queue = useTaskQueue();

const items = computed(() => props.tasks || [...queue.tasks.value].reverse());
</script>

<template>
  <div class="min-h-0 overflow-y-auto">
    <TaskEmpty
        v-if="!items.length"
        :description="emptyDescription"
        :title="emptyTitle"
    />
    <div v-else class="space-y-2">
      <TaskCard
          v-for="task in items"
          :key="task.id"
          :task="task"
          @selected="emit('selected')"
      />
    </div>
  </div>
</template>
