<script lang="ts" setup>
import type { ResourceItem } from "~/types/engine";
import ResourceViewer from "~/components/result/ResourceViewer.vue";
import ResourceInspector from "~/components/result/ResourceInspector.vue";

const props = defineProps<{
  resource: ResourceItem;
  sourceKey: string;
  resourceIndex?: number;
  resourceCount?: number;
}>();
</script>

<template>
  <div class="grid min-h-full bg-default lg:grid-cols-[minmax(0,1fr)_20rem]"
       data-testid="resource-detail">
    <ResourceViewer :resource="resource"/>
    <ResourceInspector
        :resource="resource"
        :resource-count="props.resourceCount || 1"
        :resource-index="props.resourceIndex || 0"
        :source-key="sourceKey"
    />
  </div>
</template>
