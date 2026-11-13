<script lang="ts" setup>
import type { ResourceItem } from "~/types/engine";
import {
  displayDimensions,
  formatSummary,
  resourceKindLabel
} from "~/utils/resource-format";
import ResourceViewer from "~/components/result/ResourceViewer.vue";
import VariantList from "~/components/result/VariantList.vue";

const props = defineProps<{
  resources: ResourceItem[];
  sourceKey: string;
  index: number;
}>();

const emit = defineEmits<{ "update:index": [index: number] }>();
const open = defineModel<boolean>("open", { default: false });
const variantsOpen = ref(false);
const mobileQuery = import.meta.client
    ? window.matchMedia("(max-width: 639px)")
    : null;
const isMobile = ref(mobileQuery?.matches ?? false);
const toast = useToast();
const actions = useResourceActions();
const downloads = useDownloadManager();

const handleMobileChange = (event: MediaQueryListEvent) => {
  isMobile.value = event.matches;
};

onMounted(() => mobileQuery?.addEventListener("change", handleMobileChange));
onBeforeUnmount(() => mobileQuery?.removeEventListener("change", handleMobileChange));

const resource = computed(() => props.resources[props.index]);
const resourceCount = computed(() => props.resources.length);
const multiple = computed(() => resourceCount.value > 1);
const preferredIndex = computed(() => {
  const index = resource.value?.variants.findIndex(variant => variant.url
      === resource.value?.preferredUrl);
  return index !== undefined && index >= 0
      ? index
      : undefined;
});
const preferred = computed(() => preferredIndex.value === undefined
    ? undefined
    : resource.value?.variants[preferredIndex.value]);
const displayVariant = computed(() => preferred.value || resource.value?.variants[0]);
const downloadKey = computed(() => downloads.keyFor(
    props.sourceKey,
    resource.value?.id || "",
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

const go = (delta: number) => {
  if (!multiple.value) {
    return;
  }
  const next = (props.index + delta + resourceCount.value) % resourceCount.value;
  emit("update:index", next);
};

const download = async () => {
  if (!resource.value) {
    return;
  }
  const result = await downloads.download(props.sourceKey, resource.value, {
    resourceCount: resourceCount.value,
    resourceIndex: props.index,
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

const handleKeydown = (event: KeyboardEvent) => {
  if (!open.value) {
    return;
  }
  if (event.key === "ArrowLeft") {
    go(-1);
  } else if (event.key === "ArrowRight") {
    go(1);
  }
};

onMounted(() => window.addEventListener("keydown", handleKeydown));
onBeforeUnmount(() => window.removeEventListener("keydown", handleKeydown));
</script>

<template>
  <UModal
      v-model:open="open"
      :dismissible="true"
      :fullscreen="isMobile"
      :ui="{
        content: 'overflow-hidden',
        body: 'p-0',
        footer: 'p-3 sm:px-4'
      }"
  >
    <template #body>
      <div v-if="resource" class="flex flex-col">
        <div class="relative flex h-[52dvh] items-center justify-center bg-muted sm:h-[60vh]">
          <ResourceViewer
              :key="resource.id"
              :resource="resource"
              class="h-full! max-h-none! min-h-0! size-full"
          />

          <template v-if="multiple">
            <UButton
                aria-label="Previous resource"
                class="absolute left-2 top-1/2 -translate-y-1/2"
                color="neutral"
                icon="i-lucide-chevron-left"
                variant="solid"
                @click="go(-1)"
            />
            <UButton
                aria-label="Next resource"
                class="absolute right-2 top-1/2 -translate-y-1/2"
                color="neutral"
                icon="i-lucide-chevron-right"
                variant="solid"
                @click="go(1)"
            />
            <span
                class="absolute right-3 top-3 rounded-full bg-black/55 px-2.5 py-1 text-xs font-medium text-white backdrop-blur-sm"
            >
              {{ index + 1 }} / {{ resourceCount }}
            </span>
          </template>
        </div>

        <div class="flex flex-col gap-3 p-3 sm:flex-row sm:items-center sm:justify-between sm:p-4">
          <div class="min-w-0">
            <div class="flex items-center gap-2">
              <h2 class="truncate text-sm font-semibold text-highlighted">
                {{ resourceKindLabel(resource.kind) }}
              </h2>
              <span class="text-xs text-muted">{{ displayDimensions(resource.width, resource.height) }}</span>
            </div>
            <p v-if="formatSummary(displayVariant)" class="mt-1 truncate text-xs text-muted">
              {{ formatSummary(displayVariant) }}
            </p>
          </div>

          <div class="flex shrink-0 items-center gap-2">
            <UButton
                :disabled="busy"
                :icon="busy ? 'i-lucide-loader-circle' : 'i-lucide-download'"
                :label="downloadLabel"
                :loading="busy"
                class="min-h-10"
                @click="download"
            />
            <UButton
                class="min-h-10"
                color="neutral"
                icon="i-lucide-external-link"
                label="Open"
                variant="outline"
                @click="actions.open(resource.preferredUrl)"
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
          </div>
        </div>
      </div>
    </template>
  </UModal>

  <UModal
      v-model:open="variantsOpen"
      description="Choose an available representation of this resource."
      title="Resource variants"
  >
    <template #body>
      <VariantList
          v-if="resource"
          :resource="resource"
          :resource-count="resourceCount"
          :resource-index="index"
          :source-key="sourceKey"
      />
    </template>
  </UModal>
</template>
