<script lang="ts" setup>
const open = defineModel<boolean>("open", { default: false });
const emit = defineEmits<{ added: [] }>();
const value = ref("");
const busy = ref(false);
const queue = useTaskQueue();
const toast = useToast();

const lines = computed(() => value.value.split(/\r?\n/).map(line => line.trim()).filter(Boolean));

const submit = async () => {
  if (!lines.value.length || busy.value) {
    return;
  }
  busy.value = true;
  try {
    const result = await queue.addMany(lines.value);
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
      value.value = "";
      open.value = false;
      emit("added");
    } else if (result.duplicates) {
      toast.add({
        title: "Tasks already exist",
        description: `${result.duplicates} duplicate inputs were skipped.`,
        color: "neutral"
      });
      open.value = false;
      emit("added");
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
  <UModal v-model:open="open"
          description="Enter one supported link per line."
          title="Add multiple tasks">
    <template #body>
      <UTextarea
          v-model="value"
          :rows="9"
          aria-label="Links"
          autoresize
          class="w-full"
          placeholder="One link per line"
      />
      <div class="mt-2 text-right text-xs text-muted">{{ lines.length }} {{
          lines.length === 1
              ? "task"
              : "tasks"
        }}
      </div>
    </template>
    <template #footer>
      <div class="flex w-full justify-end gap-2">
        <UButton color="neutral" label="Cancel" variant="ghost" @click="open = false"/>
        <UButton
            :disabled="!lines.length"
            :label="`Add ${lines.length || ''} ${lines.length === 1 ? 'task' : 'tasks'}`.replace('  ', ' ')"
            :loading="busy"
            icon="i-lucide-list-plus"
            @click="submit"
        />
      </div>
    </template>
  </UModal>
</template>
