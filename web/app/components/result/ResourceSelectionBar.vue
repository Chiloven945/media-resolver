<script lang="ts" setup>
const props = defineProps<{
  count: number;
  total: number;
}>();

const emit = defineEmits<{
  selectAll: [];
  clear: [];
  download: [];
  cancel: [];
}>();
</script>

<template>
  <div
      class="shrink-0 border-t border-default bg-default/95 px-3 py-3 pb-[calc(0.75rem+env(safe-area-inset-bottom))] backdrop-blur sm:px-4"
      data-testid="resource-selection-bar"
  >
    <div class="flex items-center justify-between gap-3">
      <div class="min-w-0">
        <div class="text-sm font-medium text-highlighted">{{ props.count }} selected</div>
        <div class="text-xs text-muted">of {{ props.total }} resources</div>
      </div>
      <div class="flex shrink-0 items-center gap-2">
        <UButton
            :label="props.count === props.total ? 'Clear' : 'Select all'"
            class="min-h-10"
            color="neutral"
            size="sm"
            variant="ghost"
            @click="props.count === props.total ? emit('clear') : emit('selectAll')"
        />
        <UButton
            :disabled="props.count === 0"
            class="min-h-10"
            icon="i-lucide-download"
            :label="`Download ${props.count || ''}`.trim()"
            size="sm"
            @click="emit('download')"
        />
        <UButton
            aria-label="Finish selecting"
            class="min-h-10"
            color="neutral"
            icon="i-lucide-x"
            size="sm"
            variant="ghost"
            @click="emit('cancel')"
        />
      </div>
    </div>
  </div>
</template>
