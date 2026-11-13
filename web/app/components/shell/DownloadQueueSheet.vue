<script lang="ts" setup>
const open = defineModel<boolean>("open", { default: false });
const { state, active, cancel, reset } = useBatchDownload();

const progress = computed(() => state.value.total
    ? Math.min(100, Math.round((state.value.completed / state.value.total) * 100))
    : undefined);

const label = computed(() => {
  switch (state.value.state) {
    case "preparing":
      return "Preparing downloads…";
    case "downloading":
      return state.value.current
          ? `Downloading ${state.value.current}`
          : "Downloading…";
    case "packaging":
      return "Creating ZIP archive…";
    case "completed":
      return state.value.failed
          ? `${state.value.completed} downloaded · ${state.value.failed} failed`
          : `${state.value.completed} downloaded`;
    case "failed":
      return state.value.message || "Batch download failed.";
    case "cancelled":
      return "Batch download cancelled.";
    default:
      return "No active downloads";
  }
});

const close = () => {
  open.value = false;
};

const clear = () => {
  reset();
  open.value = false;
};
</script>

<template>
  <UModal
      v-model:open="open"
      :dismissible="true"
      description="Track the current batch without leaving your workspace."
      title="Downloads"
      :ui="{ body: 'space-y-4' }"
  >
    <template #body>
      <div v-if="state.state === 'idle'" class="py-6 text-center">
        <UIcon class="mx-auto size-7 text-muted" name="i-lucide-download"/>
        <div class="mt-3 text-sm font-medium text-highlighted">Nothing downloading</div>
        <p class="mx-auto mt-1 max-w-xs text-xs leading-5 text-muted">
          Start a batch download from a task or the tasks screen.
        </p>
      </div>

      <template v-else>
        <div class="flex items-center justify-between gap-3">
          <div class="min-w-0">
            <div class="truncate text-sm font-medium text-highlighted">{{ label }}</div>
            <div class="mt-1 text-xs text-muted">
              {{ state.completed }} / {{ state.total }} complete
              <template v-if="state.failed"> · {{ state.failed }} failed</template>
            </div>
          </div>
          <span class="shrink-0 text-sm tabular-nums text-muted">
            {{ progress === undefined ? "" : `${progress}%` }}
          </span>
        </div>
        <UProgress
            :model-value="state.state === 'packaging' ? 100 : progress"
            size="sm"
        />
        <p v-if="state.state === 'failed'" class="text-sm text-error">
          {{ state.message || "Batch download failed." }}
        </p>
      </template>
    </template>

    <template #footer>
      <div class="flex w-full items-center justify-end gap-2">
        <template v-if="active">
          <UButton color="neutral" label="Hide" variant="ghost" @click="close"/>
          <UButton
              color="neutral"
              icon="i-lucide-circle-stop"
              label="Cancel batch"
              variant="outline"
              @click="cancel"
          />
        </template>
        <template v-else-if="state.state !== 'idle'">
          <UButton color="neutral" label="Clear" variant="ghost" @click="clear"/>
          <UButton label="Close" @click="open = false"/>
        </template>
        <UButton v-else label="Close" @click="open = false"/>
      </div>
    </template>
  </UModal>
</template>
