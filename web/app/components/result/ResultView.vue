<script lang="ts" setup>
import type { BatchEntry, BatchGroup } from "~/composables/useBatchDownload";
import { batchEntriesForTask } from "~/composables/useBatchDownload";
import type { ResolveTask } from "~/types/task";
import { getPublicErrorMessage } from "~/types/task";
import ResourceDetail from "~/components/result/ResourceDetail.vue";
import ResourceGrid from "~/components/result/ResourceGrid.vue";
import ResourceSelectionBar from "~/components/result/ResourceSelectionBar.vue";
import ResourceViewerModal from "~/components/result/ResourceViewerModal.vue";
import BatchDownloadDialog from "~/components/task/BatchDownloadDialog.vue";
import ManualResponseModal from "~/components/result/ManualResponseModal.vue";

const props = defineProps<{ task: ResolveTask | null }>();
const queue = useTaskQueue();
const selection = useResourceSelection();
const manualResponseOpen = ref(false);
const batchOpen = ref(false);
const batchGroups = ref<BatchGroup[]>([]);
const batchArchiveName = ref("media.zip");
const viewerOpen = ref(false);
const viewerIndex = ref(0);
const scrollArea = ref<HTMLElement | null>(null);

const errorMessage = computed(() => getPublicErrorMessage(props.task?.error?.code));
const resourceCount = computed(() => props.task?.result?.resources.length || 0);
const resources = computed(() => props.task?.result?.resources || []);
const multiResource = computed(() => resourceCount.value > 1);
const selectedResources = computed(() => resources.value.filter(resource =>
    selection.isSelected(resource.id)
));
const recoveryAvailable = computed(() => props.task
    ? queue.canRecover(props.task.id)
    : false);

watch(() => props.task?.id, async () => {
  manualResponseOpen.value = false;
  batchOpen.value = false;
  viewerOpen.value = false;
  viewerIndex.value = 0;
  selection.reset();
  await nextTick();
  scrollArea.value?.scrollTo({ top: 0 });
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

const openBatch = (entries: BatchEntry[], archiveName: string) => {
  if (!entries.length || !props.task) {
    return;
  }
  batchGroups.value = [{
    id: props.task.id,
    title: props.task.sourceKey || `Task ${props.task.sequence}`,
    subtitle: `${entries.length} ${entries.length === 1 ? "resource" : "resources"}`,
    entries
  }];
  batchArchiveName.value = archiveName;
  batchOpen.value = true;
};

const downloadAll = () => {
  if (!props.task?.result) {
    return;
  }
  openBatch(
      batchEntriesForTask(props.task),
      `${props.task.result.sourceKey || `task-${props.task.sequence}`}.zip`
  );
};

const downloadSelected = () => {
  if (!props.task?.result) {
    return;
  }

  const entries = selectedResources.value.map(resource => ({
    sourceKey: props.task!.result!.sourceKey,
    resource,
    resourceIndex: resources.value.findIndex(item => item.id === resource.id),
    resourceCount: resources.value.length
  })).filter(entry => entry.resourceIndex >= 0);

  openBatch(
      entries,
      `${props.task.result.sourceKey || `task-${props.task.sequence}`}.zip`
  );
};

const selectAll = () => {
  selection.select(resources.value.map(resource => resource.id));
};

const openResource = (index: number) => {
  viewerIndex.value = index;
  viewerOpen.value = true;
};
</script>

<template>
  <ResultEmpty v-if="!task"/>

  <div v-else class="flex h-full min-h-0 flex-col overflow-hidden">
    <ResultHeader
        :completed-label="completedLabel"
        :resource-count="resourceCount"
        :task="task"
    />

    <div
        ref="scrollArea"
        class="min-h-0 flex-1 overflow-y-auto overscroll-contain"
        data-testid="result-scroll-area"
    >
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
          :resource-count="task.result.resources.length"
          :resource-index="0"
          :source-key="task.result.sourceKey"
      />

      <div v-else-if="task.result" class="p-3 sm:p-5 lg:p-6">
        <ResourceGrid
            :resources="task.result.resources"
            :selected-ids="selection.selectedIds.value"
            :selecting="selection.selecting.value"
            :source-key="task.result.sourceKey"
            @download-all="downloadAll"
            @open-resource="openResource"
            @select-all="selectAll"
            @start-selecting="selection.start"
            @stop-selecting="selection.stop"
            @toggle-resource="selection.toggle"
        />
      </div>
    </div>

    <div
        v-if="multiResource && !selection.selecting.value"
        class="shrink-0 border-t border-default bg-default/95 px-3 py-3 pb-[calc(0.75rem+env(safe-area-inset-bottom))] backdrop-blur lg:hidden"
    >
      <div class="flex items-center gap-2">
        <UButton
            class="min-h-10 flex-1"
            icon="i-lucide-download"
            label="Download all"
            @click="downloadAll"
        />
        <UButton
            class="min-h-10"
            color="neutral"
            icon="i-lucide-square-check"
            label="Select"
            variant="outline"
            @click="selection.start"
        />
      </div>
    </div>

    <ResourceSelectionBar
        v-if="multiResource && selection.selecting.value"
        :count="selection.selectedIds.value.length"
        :total="resourceCount"
        @cancel="selection.stop"
        @clear="selection.clear"
        @download="downloadSelected"
        @select-all="selectAll"
    />
  </div>

  <BatchDownloadDialog
      v-model:open="batchOpen"
      :archive-name="batchArchiveName"
      :groups="batchGroups"
      :title="task?.result?.resources.length === 1 ? 'Download resource' : 'Download resources'"
      description="Choose how to save the selected resources."
  />

  <ResourceViewerModal
      v-if="task?.result"
      v-model:open="viewerOpen"
      v-model:index="viewerIndex"
      :resources="resources"
      :source-key="task.result.sourceKey"
  />

  <ManualResponseModal
      v-if="task"
      v-model:open="manualResponseOpen"
      :task-id="task.id"
  />
</template>
