<script lang="ts" setup>
import type { ResourceItem } from "~/types/engine";
import ResourceCard from "~/components/result/ResourceCard.vue";

const props = defineProps<{
  resources: ResourceItem[];
  sourceKey: string;
}>();
</script>

<template>
  <div class="grid grid-cols-1 gap-4 md:grid-cols-2 2xl:grid-cols-3" data-testid="resource-grid">
    <ResourceCard
        v-for="(resource, index) in props.resources"
        :key="resource.id"
        :resource="resource"
        :resource-index="index"
        :source-key="sourceKey"
    />
  </div>
</template>
