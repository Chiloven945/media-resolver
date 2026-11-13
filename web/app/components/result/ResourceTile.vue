<script lang="ts" setup>
import type { ResourceItem } from "~/types/engine";
import { displayDimensions, resourceKindLabel } from "~/utils/resource-format";

const props = withDefaults(defineProps<{
  resource: ResourceItem;
  resourceIndex: number;
  selecting?: boolean;
  selected?: boolean;
}>(), {
  selecting: false,
  selected: false
});

const emit = defineEmits<{
  open: [];
  toggleSelection: [];
}>();

const previewUrl = computed(() => props.resource.previewUrl || props.resource.preferredUrl);
const label = computed(() => resourceKindLabel(props.resource.kind));
const dimensions = computed(() => displayDimensions(props.resource.width, props.resource.height));
const playable = computed(() => props.resource.kind === "video"
    || props.resource.kind === "animation");

const activate = () => {
  if (props.selecting) {
    emit("toggleSelection");
    return;
  }
  emit("open");
};
</script>

<template>
  <div
      :class="selected ? 'ring-2 ring-primary ring-offset-2 ring-offset-default' : ''"
      :role="selecting ? 'checkbox' : 'button'"
      :aria-checked="selecting ? selected : undefined"
      :aria-label="`${label} ${resourceIndex + 1}`"
      class="group relative aspect-square cursor-pointer overflow-hidden rounded-xl border border-default bg-elevated text-left outline-none transition-all focus-visible:ring-2 focus-visible:ring-primary"
      data-testid="resource-tile"
      tabindex="0"
      @click="activate"
      @keydown.enter.prevent="activate"
      @keydown.space.prevent="activate"
  >
    <img
        v-if="previewUrl"
        :src="previewUrl"
        :alt="label"
        class="size-full object-cover transition-transform duration-300 group-hover:scale-[1.03]"
        loading="lazy"
        referrerpolicy="no-referrer"
    >
    <div v-else class="flex size-full items-center justify-center text-muted">
      <UIcon class="size-8" :name="playable ? 'i-lucide-film' : 'i-lucide-image'"/>
    </div>

    <div class="pointer-events-none absolute inset-0 bg-gradient-to-t from-black/70 via-black/0 to-black/20"/>

    <span
        v-if="playable"
        class="pointer-events-none absolute left-1/2 top-1/2 flex size-10 -translate-x-1/2 -translate-y-1/2 items-center justify-center rounded-full bg-black/55 text-white backdrop-blur-sm"
    >
      <UIcon class="size-5" name="i-lucide-play"/>
    </span>

    <span
        class="pointer-events-none absolute right-1.5 top-1.5 rounded-md bg-black/55 px-1.5 py-0.5 text-[10px] font-medium uppercase tracking-wide text-white backdrop-blur-sm"
    >
      {{ resource.variants.length > 1 ? `${resource.variants.length} variants` : label }}
    </span>

    <div class="pointer-events-none absolute inset-x-0 bottom-0 p-2">
      <div class="truncate text-xs font-medium text-white">{{ label }}</div>
      <div class="mt-0.5 truncate text-[10px] text-white/70">{{ dimensions }}</div>
    </div>

    <span
        v-if="selecting"
        :class="selected ? 'border-primary bg-primary text-white' : 'border-white/70 bg-black/25'"
        class="pointer-events-none absolute left-1.5 top-1.5 flex size-6 items-center justify-center rounded-md border backdrop-blur-sm"
    >
      <UIcon v-if="selected" class="size-3.5" name="i-lucide-check"/>
    </span>
  </div>
</template>
