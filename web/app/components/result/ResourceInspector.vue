<script lang="ts" setup>
import type { ResourceItem } from "~/types/engine";
import {
  displayBitrate,
  displayCodec,
  displayDimensions,
  displayFormat,
  resourceKindLabel
} from "~/utils/resource-format";
import VariantList from "~/components/result/VariantList.vue";

const props = defineProps<{
  resource: ResourceItem;
  sourceKey: string;
  resourceIndex: number;
}>();
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

const primarySummary = computed(() => {
  if (!displayVariant.value) {
    return "";
  }

  return [
    displayFormat(displayVariant.value),
    displayVariant.value?.codec
        ? displayCodec(displayVariant.value.codec)
        : "",
    displayVariant.value?.bitrate
        ? displayBitrate(displayVariant.value.bitrate)
        : ""
  ].filter(Boolean).join(" · ");
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
</script>

<template>
  <aside class="flex min-h-0 flex-col border-t border-default bg-default lg:border-l lg:border-t-0">
    <div class="space-y-5 p-4 sm:p-5">
      <div>
        <div class="text-base font-semibold text-highlighted">
          {{ resourceKindLabel(resource.kind) }}
        </div>
        <div class="mt-1 text-sm text-muted">
          {{ displayDimensions(resource.width, resource.height) }}
        </div>
        <div v-if="primarySummary" class="mt-2 text-sm text-muted">{{ primarySummary }}</div>
      </div>

      <div class="space-y-2">
        <div class="flex gap-2">
          <UButton
              :disabled="busy"
              :icon="busy ? 'i-lucide-loader-circle' : 'i-lucide-download'"
              :label="downloadLabel"
              :loading="busy"
              class="flex-1"
              @click="startDownload"
          />
          <UButton
              v-if="busy"
              aria-label="Cancel download"
              color="neutral"
              icon="i-lucide-x"
              variant="outline"
              @click="downloads.cancel(downloadKey)"
          />
        </div>

        <UProgress
            v-if="downloadState.state === 'downloading'"
            :animation="downloadState.progress === undefined ? 'carousel' : undefined"
            :model-value="downloadState.progress"
            size="xs"
        />

        <UButton
            class="w-full justify-center"
            color="neutral"
            icon="i-lucide-external-link"
            label="Open externally"
            variant="outline"
            @click="actions.open(resource.preferredUrl)"
        />
      </div>

      <div
          v-if="downloadState.state === 'failed'"
          class="border border-error/20 bg-error/5 p-3"
      >
        <div class="text-sm font-medium text-highlighted">Download unavailable</div>
        <p class="mt-1 text-xs leading-5 text-muted">Direct and managed download methods could not
          complete this resource.</p>
        <UButton
            class="mt-2"
            color="neutral"
            icon="i-lucide-external-link"
            label="Open resource"
            size="sm"
            variant="ghost"
            @click="actions.open(resource.preferredUrl)"
        />
      </div>

      <div v-if="resource.variants.length" class="border-t border-default pt-4">
        <div class="mb-1 flex items-center justify-between gap-3">
          <h3 class="text-sm font-semibold text-highlighted">Variants</h3>
          <span class="text-xs tabular-nums text-muted">{{ resource.variants.length }}</span>
        </div>
        <VariantList
            :resource="resource"
            :resource-index="resourceIndex"
            :source-key="sourceKey"
            inline
        />
      </div>
    </div>
  </aside>
</template>
