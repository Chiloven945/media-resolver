<script setup lang="ts">
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
          title="Add multiple tasks"
          description="Enter one supported link per line.">
    <template #body>
      <UTextarea
          v-model="value"
          :rows="9"
          autoresize
          placeholder="One link per line"
          class="w-full"
          aria-label="Links"
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
        <UButton label="Cancel" color="neutral" variant="ghost" @click="open = false"/>
        <UButton
            :label="`Add ${lines.length || ''} ${lines.length === 1 ? 'task' : 'tasks'}`.replace('  ', ' ')"
            icon="i-lucide-list-plus"
            :loading="busy"
            :disabled="!lines.length"
            @click="submit"
        />
      </div>
    </template>
  </UModal>
</template>
