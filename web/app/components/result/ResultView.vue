<script lang="ts" setup>
import type { ResolveTask } from "~/types/task";
import { getPublicErrorMessage } from "~/types/task";
import ResourceDetail from "~/components/result/ResourceDetail.vue";
import ResourceGrid from "~/components/result/ResourceGrid.vue";
import ManualResponseModal from "~/components/result/ManualResponseModal.vue";

const props = defineProps<{ task: ResolveTask | null }>();
const queue = useTaskQueue();
const manualResponseOpen = ref(false);

const errorMessage = computed(() => getPublicErrorMessage(props.task?.error?.code));
const resourceCount = computed(() => props.task?.result?.resources.length || 0);
const recoveryAvailable = computed(() => props.task
    ? queue.canRecover(props.task.id)
    : false);

watch(() => props.task?.id, () => {
  manualResponseOpen.value = false;
});

const completedLabel = computed(() => {
  if (!props.task?.completedAt) {
    return "";
  }
  const seconds = Math.max(
      0,
      Math.round((
          Date.now() - props.task.completedAt
      ) / 1000)
  );
  if (seconds < 10) {
    return "Completed just now";
  }
  if (seconds < 60) {
    return `Completed ${seconds}s ago`;
  }
  const minutes = Math.round(seconds / 60);
  return `Completed ${minutes}m ago`;
});
</script>

<template>
  <ResultEmpty v-if="!task"/>

  <div v-else class="flex h-full min-h-0 flex-col overflow-hidden">
    <ResultHeader
        :completed-label="completedLabel"
        :resource-count="resourceCount"
        :task="task"
    />

    <div class="min-h-0 flex-1 overflow-y-auto overscroll-contain" data-testid="result-scroll-area">
      <ResultStatus
          v-if="task.state !== 'ready'"
          :error-message="errorMessage"
          :recovery-available="recoveryAvailable"
          :task="task"
          @cancel="queue.cancel(task.id)"
          @recover="manualResponseOpen = true"
          @retry="queue.retry(task.id)"
      />

      <ResourceDetail
          v-else-if="task.result && task.result.resources.length === 1 && task.result.resources[0]"
          :resource="task.result.resources[0]"
          :resource-index="0"
          :source-key="task.result.sourceKey"
      />

      <div v-else-if="task.result" class="p-4 sm:p-5 lg:p-6">
        <ResourceGrid
            :resources="task.result.resources"
            :source-key="task.result.sourceKey"
        />
      </div>
    </div>
  </div>

  <ManualResponseModal
      v-if="task"
      v-model:open="manualResponseOpen"
      :task-id="task.id"
  />
</template>
