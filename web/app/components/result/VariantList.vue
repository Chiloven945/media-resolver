<script lang="ts" setup>
import type { DropdownMenuItem, TableColumn } from "@nuxt/ui";
import type { ResourceItem, ResourceVariant } from "~/types/engine";
import {
  displayBitrate,
  displayBytes,
  displayCodec,
  displayDimensions,
  displayFormat
} from "~/utils/resource-format";

const props = withDefaults(defineProps<{
  resource: ResourceItem;
  sourceKey: string;
  resourceIndex: number;
  inline?: boolean;
}>(), {
  inline: false
});
const toast = useToast();
const actions = useResourceActions();
const downloads = useDownloadManager();

interface VariantRow {
  index: number;
  quality: string;
  format: string;
  codec: string;
  bitrate: string;
  size: string;
  variant: ResourceVariant;
}

const rows = computed<VariantRow[]>(() => props.resource.variants.map((variant, index) => (
    {
      index,
      quality: displayDimensions(variant.width, variant.height).replace("Original size", "Default"),
      format: displayFormat(variant),
      codec: displayCodec(variant.codec),
      bitrate: displayBitrate(variant.bitrate),
      size: displayBytes(variant.sizeBytes),
      variant
    }
)));

const columns: TableColumn<VariantRow>[] = [
  {
    accessorKey: "quality",
    header: "Quality"
  },
  {
    accessorKey: "format",
    header: "Format"
  },
  {
    accessorKey: "codec",
    header: "Codec"
  },
  {
    accessorKey: "bitrate",
    header: "Bitrate"
  },
  {
    accessorKey: "size",
    header: "Size"
  },
  {
    id: "actions",
    header: "Action"
  }
];

const accordionItems = computed(() => rows.value.map(row => (
    {
      label: `${row.quality} · ${row.format}`,
      value: String(row.index),
      row
    }
)));

const downloadKey = (row: VariantRow) => downloads.keyFor(
    props.sourceKey,
    props.resource.id,
    row.index
);
const downloadState = (row: VariantRow) => downloads.stateFor(downloadKey(row));
const isBusy = (row: VariantRow) => ["preparing", "downloading"].includes(downloadState(row).state);
const downloadLabel = (row: VariantRow) => {
  const state = downloadState(row);

  switch (state.state) {
    case "preparing":
      return "Preparing…";
    case "downloading":
      return state.progress === undefined
          ? "Downloading…"
          : `Downloading ${state.progress}%`;
    case "completed":
      return "Downloaded";
    default:
      return "Download";
  }
};

const downloadVariant = async (row: VariantRow) => {
  const result = await downloads.download(props.sourceKey, props.resource, {
    resourceIndex: props.resourceIndex,
    variant: row.variant,
    variantIndex: row.index
  });

  switch (result) {
    case "downloaded":
      toast.add({ title: "Resource downloaded", color: "success" });
      break;
    case "failed":
      toast.add({
        title: "Download unavailable",
        description: "Use Open externally to access this representation.",
        color: "error"
      });
      break;
  }
};

const menuFor = (row: VariantRow): DropdownMenuItem[][] => [
  [
    {
      label: "Open externally",
      icon: "i-lucide-external-link",
      onSelect: () => actions.open(row.variant.url)
    }
  ]
];
</script>

<template>
  <template v-if="inline">
    <div class="hidden divide-y divide-default lg:block">
      <div
          v-for="row in rows"
          :key="row.index"
          class="flex items-center gap-2 py-2.5"
      >
        <div class="min-w-0 flex-1">
          <div class="truncate text-sm font-medium text-highlighted">{{ row.quality }}</div>
          <div class="mt-0.5 truncate text-xs text-muted">{{ row.format }} · {{ row.bitrate }}</div>
        </div>
        <UTooltip :text="downloadLabel(row)">
          <UButton
              :aria-label="`Download ${row.quality}`"
              :class="{ 'animate-pulse': isBusy(row) }"
              :disabled="isBusy(row)"
              :icon="isBusy(row) ? 'i-lucide-loader-circle' : 'i-lucide-download'"
              color="primary"
              size="sm"
              variant="ghost"
              @click="downloadVariant(row)"
          />
        </UTooltip>
        <UDropdownMenu :items="menuFor(row)">
          <UButton
              :aria-label="`More actions for ${row.quality}`"
              color="neutral"
              icon="i-lucide-ellipsis"
              size="sm"
              variant="ghost"
          />
        </UDropdownMenu>
      </div>
    </div>

    <UAccordion :items="accordionItems" class="lg:hidden">
      <template #body="{ item }">
        <div class="space-y-3 px-1 pb-2">
          <div class="grid grid-cols-2 gap-x-4 gap-y-2 text-sm">
            <span class="text-muted">Codec</span>
            <span class="text-right font-medium text-highlighted">{{ item.row.codec }}</span>
            <span class="text-muted">Bitrate</span>
            <span class="text-right font-medium text-highlighted">{{ item.row.bitrate }}</span>
            <span class="text-muted">Size</span>
            <span class="text-right font-medium text-highlighted">{{ item.row.size }}</span>
          </div>
          <div class="grid grid-cols-1 gap-2 sm:grid-cols-2">
            <UButton
                :disabled="isBusy(item.row)"
                :icon="isBusy(item.row) ? 'i-lucide-loader-circle' : 'i-lucide-download'"
                :label="downloadLabel(item.row)"
                :loading="isBusy(item.row)"
                @click="downloadVariant(item.row)"
            />
            <UButton
                color="neutral"
                icon="i-lucide-external-link"
                label="Open externally"
                variant="outline"
                @click="actions.open(item.row.variant.url)"
            />
          </div>
        </div>
      </template>
    </UAccordion>
  </template>

  <template v-else>
    <div class="hidden sm:block">
      <UTable :columns="columns" :data="rows">
        <template #actions-cell="{ row }">
          <div class="flex justify-end gap-1">
            <UTooltip :text="downloadLabel(row.original)">
              <UButton
                  :aria-label="`Download ${row.original.quality}`"
                  :disabled="isBusy(row.original)"
                  :icon="isBusy(row.original) ? 'i-lucide-loader-circle' : 'i-lucide-download'"
                  color="primary"
                  size="sm"
                  variant="ghost"
                  @click="downloadVariant(row.original)"
              />
            </UTooltip>
            <UDropdownMenu :items="menuFor(row.original)">
              <UButton
                  :aria-label="`More actions for ${row.original.quality}`"
                  color="neutral"
                  icon="i-lucide-ellipsis"
                  size="sm"
                  variant="ghost"
              />
            </UDropdownMenu>
          </div>
        </template>
      </UTable>
    </div>

    <UAccordion :items="accordionItems" class="sm:hidden">
      <template #body="{ item }">
        <div class="space-y-3 px-1 pb-2">
          <div class="grid grid-cols-2 gap-x-4 gap-y-2 text-sm">
            <span class="text-muted">Codec</span>
            <span class="text-right font-medium text-highlighted">{{ item.row.codec }}</span>
            <span class="text-muted">Bitrate</span>
            <span class="text-right font-medium text-highlighted">{{ item.row.bitrate }}</span>
            <span class="text-muted">Size</span>
            <span class="text-right font-medium text-highlighted">{{ item.row.size }}</span>
          </div>
          <div class="grid grid-cols-1 gap-2">
            <UButton
                :disabled="isBusy(item.row)"
                :icon="isBusy(item.row) ? 'i-lucide-loader-circle' : 'i-lucide-download'"
                :label="downloadLabel(item.row)"
                :loading="isBusy(item.row)"
                @click="downloadVariant(item.row)"
            />
            <UButton
                color="neutral"
                icon="i-lucide-external-link"
                label="Open externally"
                variant="outline"
                @click="actions.open(item.row.variant.url)"
            />
          </div>
        </div>
      </template>
    </UAccordion>
  </template>
</template>
