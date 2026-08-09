<script setup lang="ts">
import type { ResolveTask } from "~/types/task";
import { getPublicErrorMessage } from "~/types/task";
import ResourceGrid from "~/components/result/ResourceGrid.vue";
import ManualResponseModal from "~/components/result/ManualResponseModal.vue";

const props = defineProps<{ task: ResolveTask | null }>();
const emit = defineEmits<{ newTask: [] }>();
const queue = useTaskQueue();
const manualResponseOpen = ref(false);
const toast = useToast();

const errorCopy = computed(() => getPublicErrorMessage(props.task?.error?.code));
const resourceCount = computed(() => props.task?.result?.resources.length || 0);

const openResponse = () => {
  if (!props.task || !queue.openResponse(props.task.id)) {
    toast.add({
      title: "Unable to open response",
      description: "Allow pop-ups for this page and try again.",
      color: "error"
    });
    return;
  }

  toast.add({
    title: "Response opened",
    description: "Copy the complete response, then return here to continue.",
    color: "neutral"
  });
};

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
  <ResultEmpty v-if="!task" @new-task="emit('newTask')"/>

  <div v-else class="mx-auto w-full max-w-7xl p-4 sm:p-6 lg:p-8">
    <div class="mb-6 flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
      <div>
        <div class="flex flex-wrap items-center gap-2">
          <h1 class="text-xl font-semibold text-highlighted sm:text-2xl">Task {{
              task.sequence
            }}</h1>
          <TaskStatus :state="task.state"/>
        </div>
        <p v-if="task.state === 'ready'" class="mt-2 text-sm text-muted">
          {{ resourceCount }} {{
            resourceCount === 1
                ? "resource"
                : "resources"
          }} found
          <span v-if="completedLabel"> · {{ completedLabel }}</span>
        </p>
        <p v-else class="mt-2 text-sm text-muted">Task state is kept only for this browser
          session.</p>
      </div>

      <div class="flex gap-2">
        <UButton
            v-if="task.state === 'connecting' || task.state === 'processing'"
            label="Cancel"
            icon="i-lucide-circle-stop"
            color="neutral"
            variant="outline"
            @click="queue.cancel(task.id)"
        />
        <UButton
            v-else-if="task.state === 'cancelled' || (task.state === 'failed' && task.error?.code !== 'browser_blocked')"
            label="Retry"
            icon="i-lucide-refresh-cw"
            @click="queue.retry(task.id)"
        />
      </div>
    </div>

    <UCard v-if="task.state === 'queued'" variant="subtle">
      <div class="flex items-center gap-4 py-6">
        <UIcon name="i-lucide-clock-3" class="size-6 text-muted"/>
        <div>
          <div class="font-medium text-highlighted">Waiting in queue</div>
          <div class="mt-1 text-sm text-muted">This task will start as soon as a processing slot is
            available.
          </div>
        </div>
      </div>
    </UCard>

    <UCard v-else-if="task.state === 'connecting' || task.state === 'processing'" variant="subtle">
      <div class="space-y-5 py-6">
        <div class="flex items-center gap-4">
          <UIcon name="i-lucide-loader-circle" class="size-6 animate-spin text-primary"/>
          <div>
            <div class="font-medium text-highlighted">
              {{
                task.state === "connecting"
                    ? "Connecting to source"
                    : "Processing response"
              }}
            </div>
            <div class="mt-1 text-sm text-muted">The task will update automatically when this stage
              completes.
            </div>
          </div>
        </div>
        <UProgress animation="carousel"/>
      </div>
    </UCard>

    <UAlert
        v-else-if="task.state === 'failed'"
        color="error"
        variant="subtle"
        icon="i-lucide-circle-alert"
        :title="errorCopy.title"
        :description="errorCopy.description"
    >
      <template #actions>
        <div v-if="task.error?.code === 'browser_blocked'" class="flex flex-wrap gap-2">
          <UButton
              label="Open response"
              icon="i-lucide-external-link"
              color="error"
              variant="soft"
              @click="openResponse"
          />
          <UButton
              label="Continue from response"
              icon="i-lucide-clipboard-paste"
              color="neutral"
              variant="soft"
              @click="manualResponseOpen = true"
          />
        </div>
        <UButton
            v-else
            label="Retry"
            icon="i-lucide-refresh-cw"
            color="error"
            variant="soft"
            @click="queue.retry(task.id)"
        />
      </template>
    </UAlert>

    <UAlert
        v-else-if="task.state === 'cancelled'"
        color="neutral"
        variant="subtle"
        icon="i-lucide-ban"
        title="Task cancelled"
        description="You can retry this task whenever you are ready."
    >
      <template #actions>
        <UButton label="Retry" icon="i-lucide-refresh-cw" color="neutral" variant="soft"
                 @click="queue.retry(task.id)"/>
      </template>
    </UAlert>

    <ResourceGrid v-else-if="task.state === 'ready' && task.result"
                  :resources="task.result.resources"/>
  </div>

  <ManualResponseModal
      v-if="task"
      v-model:open="manualResponseOpen"
      :task-id="task.id"
  />
</template>
