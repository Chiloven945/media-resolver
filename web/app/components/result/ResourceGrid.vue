<script setup lang="ts">
import type { ResourceItem } from "~/types/engine";
import ResourceCard from "~/components/result/ResourceCard.vue";

defineProps<{ resources: ResourceItem[] }>();
</script>

<template>
  <div class="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
    <ResourceCard v-for="resource in resources" :key="resource.id" :resource="resource"/>
  </div>
</template>
