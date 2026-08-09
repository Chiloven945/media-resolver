<script lang="ts" setup>
import type { DropdownMenuItem } from "@nuxt/ui";
import type { ResourceItem } from "~/types/engine";
import { displayDimensions, formatSummary, resourceKindLabel } from "~/utils/resource-format";
import VariantList from "~/components/result/VariantList.vue";
import ResourceViewer from "~/components/result/ResourceViewer.vue";

const props = defineProps<{
  resource: ResourceItem;
  sourceKey: string;
  resourceIndex: number;
}>();
const variantsOpen = ref(false);
const toast = useToast();
const actions = useResourceActions();
const downloads = useDownloadManager();

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
const displayVariant = computed(() => preferred.value || props.resource.variants[0]);
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
    resourceIndex: props.resourceIndex,
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

const menuItems = computed<DropdownMenuItem[][]>(() => [
  [
    {
      label: "Open externally",
      icon: "i-lucide-external-link",
      onSelect: () => actions.open(props.resource.preferredUrl)
    },
    ...(
        props.resource.variants.length > 1
            ? [
              {
                label: "View variants",
                icon: "i-lucide-list-tree",
                onSelect: () => {
                  variantsOpen.value = true;
                }
              }
            ]
            : []
    )
  ]
]);
</script>

<template>
  <UCard
      :ui="{ body: 'p-0 sm:p-0', footer: 'p-3 sm:px-3' }"
      class="overflow-hidden"
      data-testid="resource-card"
  >
    <ResourceViewer :resource="resource" compact/>

    <div class="space-y-2 p-3">
      <div class="flex items-start justify-between gap-3">
        <div class="min-w-0">
          <div class="truncate text-sm font-semibold text-highlighted">
            {{ resourceKindLabel(resource.kind) }} · {{
              displayDimensions(resource.width, resource.height)
            }}
          </div>
          <p v-if="formatSummary(displayVariant)" class="mt-1 truncate text-xs text-muted">
            {{ formatSummary(displayVariant) }}
          </p>
        </div>
        <span v-if="resource.variants.length > 1" class="shrink-0 text-xs text-muted">
          {{ resource.variants.length }} variants
        </span>
      </div>
      <UProgress
          v-if="downloadState.state === 'downloading'"
          :animation="downloadState.progress === undefined ? 'carousel' : undefined"
          :model-value="downloadState.progress"
          size="xs"
      />
    </div>

    <template #footer>
      <div class="flex items-center gap-2">
        <UButton
            :disabled="busy"
            :icon="busy ? 'i-lucide-loader-circle' : 'i-lucide-download'"
            :label="downloadLabel"
            :loading="busy"
            class="flex-1"
            @click="startDownload"
        />
        <UDropdownMenu :items="menuItems">
          <UButton
              aria-label="Resource actions"
              color="neutral"
              icon="i-lucide-ellipsis"
              variant="ghost"
          />
        </UDropdownMenu>
      </div>
    </template>
  </UCard>

  <UModal
      v-model:open="variantsOpen"
      description="Choose an available representation of this resource."
      title="Resource variants"
  >
    <template #body>
      <VariantList
          :resource="resource"
          :resource-index="resourceIndex"
          :source-key="sourceKey"
      />
    </template>
  </UModal>
</template>
