<script setup lang="ts">
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
  if (result.status === "added") {
    input.value = "";
    toast.add({ title: "Task added", icon: "i-lucide-plus-circle", color: "success" });
    emit("taskAdded");
  } else if (result.status === "duplicate") {
    toast.add({
      title: "Task already exists",
      description: "The existing task has been selected.",
      color: "neutral"
    });
    emit("taskAdded");
  } else {
    const copy = getPublicErrorMessage(result.errorCode);
    toast.add({ title: copy.title, description: copy.description, color: "error" });
  }
};
</script>

<template>
  <div class="space-y-2">
    <div class="flex gap-2">
      <UInput
          v-model="input"
          placeholder="Paste a supported link"
          icon="i-lucide-link"
          :disabled="engineState !== 'ready'"
          class="min-w-0 flex-1"
          aria-label="Supported link"
          @keyup.enter="submit"
      />
      <UTooltip text="Add task">
        <UButton
            icon="i-lucide-plus"
            aria-label="Add task"
            :loading="engineState === 'loading'"
            :disabled="engineState !== 'ready' || !input.trim()"
            @click="submit"
        />
      </UTooltip>
    </div>

    <div class="flex items-center justify-between">
      <UButton
          label="Batch add"
          icon="i-lucide-list-plus"
          color="neutral"
          variant="ghost"
          size="sm"
          :disabled="engineState !== 'ready'"
          @click="batchOpen = true"
      />
      <span v-if="engineState === 'loading'" class="text-xs text-muted">Initializing…</span>
      <span v-else-if="engineState === 'error'"
            class="text-xs text-error">Initialization failed</span>
    </div>
  </div>

  <BatchTaskModal v-model:open="batchOpen" @added="emit('taskAdded')"/>
</template>
