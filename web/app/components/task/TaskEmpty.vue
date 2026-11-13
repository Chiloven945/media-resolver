<script lang="ts" setup>
withDefaults(defineProps<{
  title?: string;
  description?: string;
}>(), {
  title: "No tasks yet",
  description: "Add a supported link above to start."
});
</script>

<template>
  <div class="px-3 py-10 text-center">
    <UIcon class="mx-auto size-6 text-muted" name="i-lucide-list-todo"/>
    <div class="mt-3 text-sm font-medium text-highlighted">{{ title }}</div>
    <p class="mx-auto mt-1 max-w-xs text-xs leading-5 text-muted">{{ description }}</p>
  </div>
</template>
