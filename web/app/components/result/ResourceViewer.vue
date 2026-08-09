<script lang="ts" setup>
import type { ResourceItem } from "~/types/engine";
import { isStreamVariant } from "~/utils/resource-format";

const props = withDefaults(defineProps<{ resource: ResourceItem; compact?: boolean }>(), {
  compact: false
});
const { settings } = useAppSettings();
const actions = useResourceActions();
const active = ref(false);
const playing = ref(false);
const muted = ref(true);
const video = ref<HTMLVideoElement | null>(null);

const preferredVariant = computed(() => props.resource.variants.find(
    variant => variant.url === props.resource.preferredUrl
) || props.resource.variants[0]);

const streamOnly = computed(() => isStreamVariant(
    preferredVariant.value,
    props.resource.preferredUrl
));

const aspectRatio = computed(() => {
  const width = props.resource.width || preferredVariant.value?.width;
  const height = props.resource.height || preferredVariant.value?.height;
  return width && height
      ? `${width} / ${height}`
      : undefined;
});

onMounted(() => {
  if (settings.value.autoPreview && props.resource.kind !== "image" && !streamOnly.value) {
    active.value = true;
  }
});

const startPreview = async () => {
  if (streamOnly.value) {
    return;
  }
  active.value = true;
  await nextTick();
  try {
    await video.value?.play();
    playing.value = true;
  } catch {
    playing.value = false;
  }
};

const togglePlayback = async () => {
  if (!video.value) {
    return;
  }
  if (video.value.paused) {
    await video.value.play();
    playing.value = true;
  } else {
    video.value.pause();
    playing.value = false;
  }
};

const toggleMuted = () => {
  muted.value = !muted.value;
  if (video.value) {
    video.value.muted = muted.value;
  }
};
</script>

<template>
  <div
      :data-aspect-ratio="aspectRatio || 'unknown'"
      :data-compact="compact ? 'true' : 'false'"
      class="resource-viewer-stage relative flex size-full items-center justify-center overflow-hidden bg-muted p-3 sm:p-5"
      data-testid="resource-viewer"
  >
    <img
        v-if="resource.kind === 'image'"
        :src="resource.previewUrl || resource.preferredUrl"
        :style="aspectRatio ? { aspectRatio } : undefined"
        alt="Resource preview"
        class="resource-viewer-media h-auto w-auto"
        loading="lazy"
        referrerpolicy="no-referrer"
    >

    <template v-else>
      <video
          v-if="active && !streamOnly"
          ref="video"
          :muted="muted"
          :poster="resource.previewUrl"
          :src="resource.preferredUrl"
          :style="aspectRatio ? { aspectRatio } : undefined"
          class="resource-viewer-media h-auto w-auto"
          playsinline
          preload="metadata"
          @ended="playing = false"
          @pause="playing = false"
          @play="playing = true"
      />
      <img
          v-else-if="resource.previewUrl"
          :src="resource.previewUrl"
          :style="aspectRatio ? { aspectRatio } : undefined"
          alt="Resource preview"
          class="resource-viewer-media h-auto w-auto"
          loading="lazy"
          referrerpolicy="no-referrer"
      >
      <UIcon v-else class="size-12 text-muted" name="i-lucide-film"/>

      <div class="absolute inset-x-3 bottom-3 flex items-center justify-between gap-2 sm:inset-x-4 sm:bottom-4">
        <template v-if="streamOnly">
          <span class="text-xs font-medium text-muted">Preview unavailable</span>
          <UButton
              color="neutral"
              icon="i-lucide-external-link"
              label="Open resource"
              size="sm"
              variant="solid"
              @click="actions.open(resource.preferredUrl)"
          />
        </template>
        <UButton
            v-else-if="!active"
            color="neutral"
            icon="i-lucide-play"
            label="Preview"
            size="sm"
            variant="solid"
            @click="startPreview"
        />
        <template v-else>
          <UButton
              :aria-label="playing ? 'Pause preview' : 'Play preview'"
              :icon="playing ? 'i-lucide-pause' : 'i-lucide-play'"
              color="neutral"
              size="sm"
              variant="solid"
              @click="togglePlayback"
          />
          <UButton
              :aria-label="muted ? 'Unmute preview' : 'Mute preview'"
              :icon="muted ? 'i-lucide-volume-x' : 'i-lucide-volume-2'"
              color="neutral"
              size="sm"
              variant="solid"
              @click="toggleMuted"
          />
        </template>
      </div>
    </template>
  </div>
</template>
