<script lang="ts" setup>
import { getPublicErrorMessage } from "~/types/task";

const emit = defineEmits<{ taskAdded: [count: number] }>();
const mode = ref<"single" | "batch">("single");
const input = ref("");
const batchValue = ref("");
const busy = ref(false);
const queue = useTaskQueue();
const { state: engineState } = useEngine();
const toast = useToast();

const batchLines = computed(() => batchValue.value
    .split(/\r?\n/)
    .map(line => line.trim())
    .filter(Boolean));

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
      emit("taskAdded", 1);
      break;
    case "duplicate":
      input.value = "";
      toast.add({
        title: "Task already exists",
        description: "The existing task has been selected.",
        color: "neutral"
      });
      emit("taskAdded", 1);
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

const submitBatch = async () => {
  if (!batchLines.value.length || busy.value || engineState.value !== "ready") {
    return;
  }

  busy.value = true;
  try {
    const result = await queue.addMany(batchLines.value);
    if (result.added) {
      toast.add({
        title: `${result.added} ${result.added === 1
            ? "task"
            : "tasks"} added`,
        description: [
          result.duplicates
              ? `${result.duplicates} duplicate`
              : "",
          result.invalid
              ? `${result.invalid} invalid`
              : ""
        ].filter(Boolean).join(" · ") || undefined,
        color: "success"
      });
      batchValue.value = "";
      emit("taskAdded", result.added);
    } else if (result.duplicates) {
      toast.add({
        title: "Tasks already exist",
        description: `${result.duplicates} duplicate inputs were skipped.`,
        color: "neutral"
      });
      emit("taskAdded", 0);
    } else {
      toast.add({
        title: "No tasks added",
        description: "No supported links were found.",
        color: "error"
      });
    }
  } finally {
    busy.value = false;
  }
};
</script>

<template>
  <div class="rounded-xl border border-default bg-default p-2 shadow-sm">
    <div class="flex gap-1 rounded-lg bg-elevated p-1" role="tablist" aria-label="Add mode">
      <button
          :aria-selected="mode === 'single'"
          :class="mode === 'single'
            ? 'bg-default text-highlighted shadow-sm'
            : 'text-muted hover:text-highlighted'"
          class="flex-1 rounded-md px-3 py-1.5 text-xs font-medium transition-colors"
          role="tab"
          type="button"
          @click="mode = 'single'"
      >
        Single link
      </button>
      <button
          :aria-selected="mode === 'batch'"
          :class="mode === 'batch'
            ? 'bg-default text-highlighted shadow-sm'
            : 'text-muted hover:text-highlighted'"
          class="flex-1 rounded-md px-3 py-1.5 text-xs font-medium transition-colors"
          role="tab"
          type="button"
          @click="mode = 'batch'"
      >
        Batch add
      </button>
    </div>

    <form v-if="mode === 'single'" class="mt-2 flex gap-2" @submit.prevent="submit">
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
    </form>

    <form v-else class="mt-2 space-y-2" @submit.prevent="submitBatch">
      <UTextarea
          v-model="batchValue"
          :disabled="engineState !== 'ready'"
          :rows="5"
          aria-label="Links"
          autoresize
          class="w-full"
          placeholder="One supported link per line"
      />
      <div class="flex items-center justify-between gap-2">
        <span class="text-xs text-muted">
          {{ batchLines.length }} {{ batchLines.length === 1 ? "link" : "links" }}
        </span>
        <UButton
            :disabled="engineState !== 'ready' || !batchLines.length"
            :icon="busy ? 'i-lucide-loader-circle' : 'i-lucide-list-plus'"
            :label="batchLines.length
              ? `Add ${batchLines.length} ${batchLines.length === 1 ? 'task' : 'tasks'}`
              : 'Add tasks'"
            :loading="busy"
            size="sm"
            type="submit"
        />
      </div>
    </form>

    <div class="mt-2 flex items-center justify-between text-xs">
      <span v-if="engineState === 'loading'" class="text-muted">Initializing…</span>
      <span v-else-if="engineState === 'error'" class="text-error">Initialization failed</span>
      <span v-else class="text-muted">Supported links</span>
      <span class="text-muted">{{ mode === 'single' ? 'Enter to add' : 'One link per line' }}</span>
    </div>
  </div>
</template>
