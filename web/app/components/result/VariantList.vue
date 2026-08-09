<script setup lang="ts">
import type { TableColumn } from "@nuxt/ui";
import type { ResourceVariant } from "~/types/engine";

const props = defineProps<{ variants: ResourceVariant[] }>();
const toast = useToast();
const actions = useResourceActions();

interface VariantRow {
  index: number;
  quality: string;
  format: string;
  bitrate: string;
  variant: ResourceVariant;
}

const rows = computed<VariantRow[]>(() => props.variants.map((variant, index) => ({
  index,
  quality: variant.width && variant.height ? `${variant.width}×${variant.height}` : "Default",
  format: variant.mimeType?.split("/").pop()?.toUpperCase() || "Unknown",
  bitrate: variant.bitrate ? `${(variant.bitrate / 1_000_000).toFixed(1)} Mbps` : "—",
  variant
})));

const columns: TableColumn<VariantRow>[] = [
  { accessorKey: "quality", header: "Quality" },
  { accessorKey: "format", header: "Format" },
  { accessorKey: "bitrate", header: "Bitrate" },
  { id: "actions", header: "Action" }
];

const accordionItems = computed(() => rows.value.map(row => ({
  label: `${row.quality} · ${row.format}`,
  value: String(row.index),
  row
})));

const copy = async (variant: ResourceVariant) => {
  await actions.copy(variant.url);
  toast.add({ title: "Copied", description: "Variant address copied to clipboard.", color: "success" });
};
</script>

<template>
  <div class="hidden sm:block">
    <UTable :data="rows" :columns="columns">
      <template #actions-cell="{ row }">
        <div class="flex justify-end gap-1">
          <UTooltip text="Open">
            <UButton
              icon="i-lucide-external-link"
              color="neutral"
              variant="ghost"
              size="sm"
              aria-label="Open variant"
              @click="actions.open(row.original.variant.url)"
            />
          </UTooltip>
          <UTooltip text="Copy address">
            <UButton
              icon="i-lucide-copy"
              color="neutral"
              variant="ghost"
              size="sm"
              aria-label="Copy variant address"
              @click="copy(row.original.variant)"
            />
          </UTooltip>
        </div>
      </template>
    </UTable>
  </div>

  <UAccordion :items="accordionItems" class="sm:hidden">
    <template #body="{ item }">
      <div class="space-y-3 px-1 pb-2">
        <div class="flex items-center justify-between text-sm">
          <span class="text-muted">Bitrate</span>
          <span class="font-medium text-highlighted">{{ item.row.bitrate }}</span>
        </div>
        <div class="grid grid-cols-2 gap-2">
          <UButton label="Open" icon="i-lucide-external-link" color="neutral" variant="outline"
                   @click="actions.open(item.row.variant.url)" />
          <UButton label="Copy" icon="i-lucide-copy" @click="copy(item.row.variant)" />
        </div>
      </div>
    </template>
  </UAccordion>
</template>
