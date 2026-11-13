<script lang="ts" setup>
import type { ResourceItem } from "~/types/engine";
import ResourceViewer from "~/components/result/ResourceViewer.vue";
import ResourceInspector from "~/components/result/ResourceInspector.vue";
import VariantList from "~/components/result/VariantList.vue";

const props = defineProps<{
  resource: ResourceItem;
  sourceKey: string;
  resourceIndex?: number;
  resourceCount?: number;
}>();
const toast = useToast();
const actions = useResourceActions();
const downloads = useDownloadManager();
const variantsOpen = ref(false);

const preferredIndex = computed(() => {
  const index = props.resource.variants.findIndex(variant => variant.url
      === props.resource.preferredUrl);
  return index >= 0
      ? index
      : undefined;
});
const preferred = computed(() => preferredIndex.value === undefined
    ? undefined
    : props.resource.variants[preferredIndex.value]);
const downloadKey = computed(() => downloads.keyFor(
    props.sourceKey,
    props.resource.id,
    preferredIndex.value
));
const downloadState = computed(() => downloads.stateFor(downloadKey.value));
const busy = computed(() => ["preparing", "downloading"].includes(downloadState.value.state));
const downloadLabel = computed(() => {
  if (downloadState.value.state === "preparing") {
    return "Preparing…";
  }
  if (downloadState.value.state === "downloading") {
    return downloadState.value.progress === undefined
        ? "Downloading…"
        : `Downloading ${downloadState.value.progress}%`;
  }
  if (downloadState.value.state === "completed") {
    return "Downloaded";
  }
  return "Download";
});

const startDownload = async () => {
  const result = await downloads.download(props.sourceKey, props.resource, {
    resourceCount: props.resourceCount || 1,
    resourceIndex: props.resourceIndex || 0,
    variant: preferred.value,
    variantIndex: preferredIndex.value
  });
  if (result === "downloaded") {
    toast.add({ title: "Resource downloaded", color: "success" });
  } else if (result === "failed") {
    toast.add({
      title: "Download unavailable",
      description: "Use Open externally to access this resource.",
      color: "error"
    });
  }
};
</script>

<template>
  <div class="flex min-h-full flex-col bg-default" data-testid="resource-detail">
    <div class="grid min-h-0 flex-1 lg:grid-cols-[minmax(0,1fr)_20rem]">
      <ResourceViewer :resource="resource"/>
      <ResourceInspector
          :resource="resource"
          :resource-count="props.resourceCount || 1"
          :resource-index="props.resourceIndex || 0"
          :source-key="sourceKey"
      />
    </div>

    <div
        class="sticky bottom-0 z-10 space-y-2 border-t border-default bg-default/95 px-3 py-3 pb-[calc(0.75rem+env(safe-area-inset-bottom))] backdrop-blur lg:hidden"
        data-testid="resource-detail-actionbar"
    >
      <UProgress
          v-if="downloadState.state === 'downloading'"
          :animation="downloadState.progress === undefined ? 'carousel' : undefined"
          :model-value="downloadState.progress"
          size="xs"
      />
      <div class="flex items-center gap-2">
        <UButton
            :disabled="busy"
            :icon="busy ? 'i-lucide-loader-circle' : 'i-lucide-download'"
            :label="downloadLabel"
            :loading="busy"
            class="min-h-10 flex-1"
            @click="startDownload"
        />
        <UButton
            v-if="resource.variants.length > 1"
            aria-label="Resource variants"
            class="min-h-10"
            color="neutral"
            icon="i-lucide-list-tree"
            variant="outline"
            @click="variantsOpen = true"
        />
        <UButton
            class="min-h-10"
            color="neutral"
            icon="i-lucide-external-link"
            label="Open"
            variant="outline"
            @click="actions.open(resource.preferredUrl)"
        />
      </div>
    </div>
  </div>

  <UModal
      v-model:open="variantsOpen"
      description="Choose an available representation of this resource."
      title="Resource variants"
  >
    <template #body>
      <VariantList
          :resource="resource"
          :resource-count="props.resourceCount || 1"
          :resource-index="props.resourceIndex || 0"
          :source-key="sourceKey"
      />
    </template>
  </UModal>
</template>
