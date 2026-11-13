<script lang="ts" setup>
import type { BatchGroup, BatchMode } from "~/composables/useBatchDownload";

const props = withDefaults(defineProps<{
  groups: BatchGroup[];
  archiveName: string;
  title?: string;
  description?: string;
}>(), {
  title: "Batch download",
  description: "Choose the tasks and output format."
});

const open = defineModel<boolean>("open", { default: false });
const {
  state: batchState,
  active: batchActive,
  start: startBatch,
  cancel: cancelBatch,
  reset: resetBatch
} = useBatchDownload();

const selectedIds = ref<string[]>([]);
const mode = ref<BatchMode>("files");

const selectedGroups = computed(() => props.groups.filter(group =>
    selectedIds.value.includes(group.id)
));
const selectedEntries = computed(() => selectedGroups.value.flatMap(group => group.entries));
const totalFiles = computed(() => selectedEntries.value.length);
const allSelected = computed(() => props.groups.length > 0 && props.groups.every(group =>
    selectedIds.value.includes(group.id)
));
const showProgress = computed(() => batchState.value.state !== "idle");
const progress = computed(() => {
  if (!batchState.value.total) {
    return undefined;
  }
  return Math.min(100, Math.round((
      batchState.value.completed / batchState.value.total
  ) * 100));
});

watch([open, () => props.groups], () => {
  if (!open.value || batchActive.value) {
    return;
  }
  selectedIds.value = props.groups
      .filter(group => group.entries.length > 0)
      .map(group => group.id);
  mode.value = "files";
  resetBatch();
}, { immediate: true });

const toggleGroup = (id: string, value: boolean | "indeterminate") => {
  if (value === true) {
    if (!selectedIds.value.includes(id)) {
      selectedIds.value = [...selectedIds.value, id];
    }
    return;
  }
  selectedIds.value = selectedIds.value.filter(value => value !== id);
};

const toggleAll = () => {
  selectedIds.value = allSelected.value
      ? []
      : props.groups.filter(group => group.entries.length > 0).map(group => group.id);
};

const start = () => {
  if (!selectedEntries.value.length) {
    return;
  }
  void startBatch(selectedEntries.value, mode.value, props.archiveName);
};

const close = () => {
  open.value = false;
};

const stateLabel = computed(() => {
  switch (batchState.value.state) {
    case "preparing":
      return "Preparing downloads…";
    case "downloading":
      return batchState.value.current
          ? `Downloading ${batchState.value.current}`
          : "Downloading…";
    case "packaging":
      return "Creating ZIP archive…";
    case "completed":
      return batchState.value.failed
          ? `${batchState.value.completed} downloaded · ${batchState.value.failed} failed`
          : `${batchState.value.completed} downloaded`;
    case "failed":
      return batchState.value.message || "Batch download failed.";
    case "cancelled":
      return "Batch download cancelled.";
    default:
      return "Ready to download.";
  }
});
</script>

<template>
  <UModal
      v-model:open="open"
      :dismissible="true"
      :description="description"
      :title="title"
      :ui="{ body: 'space-y-4' }"
  >
    <template #body>
      <template v-if="!showProgress">
        <div class="space-y-4">
          <div class="flex items-center justify-between gap-3">
            <div>
              <div class="text-sm font-medium text-highlighted">Tasks</div>
              <div class="mt-1 text-xs text-muted">
                {{ selectedGroups.length }} of {{ groups.length }} selected · {{ totalFiles }}
                {{ totalFiles === 1 ? "file" : "files" }}
              </div>
            </div>
            <UButton
                :label="allSelected ? 'Clear all' : 'Select all'"
                color="neutral"
                size="sm"
                variant="ghost"
                @click="toggleAll"
            />
          </div>

          <div class="max-h-64 space-y-1 overflow-y-auto rounded-lg border border-default p-2">
            <label
                v-for="group in groups"
                :key="group.id"
                class="flex cursor-pointer items-start gap-3 rounded-md p-2 hover:bg-elevated/70"
            >
              <UCheckbox
                  :model-value="selectedIds.includes(group.id)"
                  :disabled="group.entries.length === 0"
                  class="mt-0.5"
                  @update:model-value="value => toggleGroup(group.id, value)"
              />
              <span class="min-w-0 flex-1">
                <span class="block truncate text-sm font-medium text-highlighted">
                  {{ group.title }}
                </span>
                <span class="mt-0.5 block truncate text-xs text-muted">
                  {{ group.subtitle || `${group.entries.length} files` }}
                </span>
              </span>
              <span class="shrink-0 text-xs tabular-nums text-muted">
                {{ group.entries.length }}
              </span>
            </label>
          </div>

          <div>
            <div class="mb-2 text-sm font-medium text-highlighted">Output</div>
            <div class="grid grid-cols-1 gap-2 sm:grid-cols-2">
              <UButton
                  :variant="mode === 'files' ? 'solid' : 'outline'"
                  block
                  class="justify-start"
                  color="neutral"
                  icon="i-lucide-download"
                  label="Individual files"
                  @click="mode = 'files'"
              />
              <UButton
                  :variant="mode === 'zip' ? 'solid' : 'outline'"
                  block
                  class="justify-start"
                  color="neutral"
                  icon="i-lucide-file-archive"
                  label="ZIP archive"
                  @click="mode = 'zip'"
              />
            </div>
            <p v-if="mode === 'zip'" class="mt-2 text-xs leading-5 text-muted">
              The archive is prepared locally and may use significant memory for large videos.
            </p>
            <p v-else class="mt-2 text-xs leading-5 text-muted">
              Your browser may ask for permission to download multiple files.
            </p>
          </div>
        </div>
      </template>

      <div v-else class="space-y-4 py-2">
        <div class="flex items-center justify-between gap-3">
          <div class="min-w-0">
            <div class="truncate text-sm font-medium text-highlighted">{{ stateLabel }}</div>
            <div class="mt-1 text-xs text-muted">
              {{ batchState.completed }} / {{ batchState.total }} complete
              <template v-if="batchState.failed"> · {{ batchState.failed }} failed</template>
            </div>
          </div>
          <span class="shrink-0 text-sm tabular-nums text-muted">
            {{ progress === undefined ? "" : `${progress}%` }}
          </span>
        </div>
        <UProgress
            :model-value="batchState.state === 'packaging' ? 100 : progress"
            size="sm"
        />
        <p v-if="batchState.state === 'failed'" class="text-sm text-error">
          {{ batchState.message || "Batch download failed." }}
        </p>
      </div>
    </template>

    <template #footer>
      <div class="flex w-full items-center justify-end gap-2">
        <template v-if="batchActive">
          <UButton color="neutral" label="Hide" variant="ghost" @click="close"/>
          <UButton
              color="neutral"
              icon="i-lucide-circle-stop"
              label="Cancel batch"
              variant="outline"
              @click="cancelBatch"
          />
        </template>
        <template v-else-if="showProgress">
          <UButton label="Close" @click="open = false"/>
        </template>
        <template v-else>
          <UButton color="neutral" label="Cancel" variant="ghost" @click="open = false"/>
          <UButton
              :disabled="totalFiles === 0"
              :icon="mode === 'zip' ? 'i-lucide-file-archive' : 'i-lucide-download'"
              :label="mode === 'zip' ? `Download ${totalFiles} as ZIP` : `Download ${totalFiles}`"
              @click="start"
          />
        </template>
      </div>
    </template>
  </UModal>
</template>
