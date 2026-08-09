<script lang="ts" setup>
import { getPublicErrorMessage } from "~/types/task";
import BatchTaskModal from "~/components/task/BatchTaskModal.vue";

const emit = defineEmits<{ taskAdded: [] }>();
const input = ref("");
const batchOpen = ref(false);
const queue = useTaskQueue();
const { state: engineState } = useEngine();
const toast = useToast();

const submit = async () => {
  if (!input.value.trim() || engineState.value !== "ready") {
    return;
  }

  const result = await queue.add(input.value);

  switch (result.status) {
    case "added":
      input.value = "";
      toast.add({
        title: "Task added",
        icon: "i-lucide-plus-circle",
        color: "success"
      });
      emit("taskAdded");
      break;
    case "duplicate":
      input.value = "";
      toast.add({
        title: "Task already exists",
        description: "The existing task has been selected.",
        color: "neutral"
      });
      emit("taskAdded");
      break;
    default: {
      const message = getPublicErrorMessage(result.errorCode);
      toast.add({
        title: message.title,
        description: message.description,
        color: "error"
      });
      break;
    }
  }
};
</script>

<template>
  <form class="space-y-2" @submit.prevent="submit">
    <div class="flex gap-2">
      <UInput
          v-model="input"
          :disabled="engineState !== 'ready'"
          aria-label="Supported link"
          class="min-w-0 flex-1"
          icon="i-lucide-link"
          placeholder="Paste a supported link"
          size="md"
      />
      <UTooltip text="Add task">
        <UButton
            :disabled="engineState !== 'ready' || !input.trim()"
            :loading="engineState === 'loading'"
            aria-label="Add task"
            icon="i-lucide-plus"
            size="md"
            type="submit"
        />
      </UTooltip>
    </div>

    <div class="flex items-center justify-between">
      <UButton
          :disabled="engineState !== 'ready'"
          color="neutral"
          icon="i-lucide-list-plus"
          label="Batch add"
          size="sm"
          type="button"
          variant="ghost"
          @click="batchOpen = true"
      />
      <span v-if="engineState === 'loading'" class="text-xs text-muted">Initializing…</span>
      <span v-else-if="engineState === 'error'"
            class="text-xs text-error">Initialization failed</span>
    </div>
  </form>

  <BatchTaskModal v-model:open="batchOpen" @added="emit('taskAdded')"/>
</template>
