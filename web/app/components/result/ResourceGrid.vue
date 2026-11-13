<script lang="ts" setup>
import type { ResourceItem } from "~/types/engine";
import ResourceCard from "~/components/result/ResourceCard.vue";
import ResourceTile from "~/components/result/ResourceTile.vue";

const props = withDefaults(defineProps<{
  resources: ResourceItem[];
  sourceKey: string;
  selecting?: boolean;
  selectedIds?: string[];
}>(), {
  selecting: false,
  selectedIds: () => []
});

const emit = defineEmits<{
  downloadAll: [];
  startSelecting: [];
  stopSelecting: [];
  toggleResource: [id: string];
  selectAll: [];
  openResource: [index: number];
}>();
</script>

<template>
  <div class="space-y-3" data-testid="resource-grid">
    <div class="hidden flex-wrap items-center justify-between gap-2 lg:flex">
      <div class="text-xs text-muted">
        {{ props.resources.length }} {{ props.resources.length === 1 ? "resource" : "resources" }}
      </div>
      <div class="flex items-center gap-2">
        <template v-if="props.selecting">
          <UButton
              color="neutral"
              icon="i-lucide-check-check"
              label="Select all"
              size="sm"
              variant="ghost"
              @click="emit('selectAll')"
          />
          <UButton
              color="neutral"
              icon="i-lucide-x"
              label="Done"
              size="sm"
              variant="outline"
              @click="emit('stopSelecting')"
          />
        </template>
        <template v-else>
          <UButton
              color="neutral"
              icon="i-lucide-square-check"
              label="Select"
              size="sm"
              variant="outline"
              @click="emit('startSelecting')"
          />
          <UButton
              icon="i-lucide-download"
              label="Download all"
              size="sm"
              @click="emit('downloadAll')"
          />
        </template>
      </div>
    </div>

    <div class="grid grid-cols-2 gap-2 lg:hidden">
      <ResourceTile
          v-for="(resource, index) in props.resources"
          :key="`tile-${resource.id}`"
          :resource="resource"
          :resource-index="index"
          :selected="props.selectedIds.includes(resource.id)"
          :selecting="props.selecting"
          @open="emit('openResource', index)"
          @toggle-selection="emit('toggleResource', resource.id)"
      />
    </div>

    <div
        class="hidden gap-3 lg:grid"
        style="grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));"
    >
      <ResourceCard
          v-for="(resource, index) in props.resources"
          :key="resource.id"
          :resource="resource"
          :resource-count="props.resources.length"
          :resource-index="index"
          :selected="props.selectedIds.includes(resource.id)"
          :selecting="props.selecting"
          :source-key="sourceKey"
          @toggle-selection="emit('toggleResource', resource.id)"
      />
    </div>
  </div>
</template>
