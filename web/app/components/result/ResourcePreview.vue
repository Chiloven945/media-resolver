<script setup lang="ts">
import type { ResourceItem } from "~/types/engine";

const props = defineProps<{ resource: ResourceItem }>();
const { settings } = useAppSettings();
const active = ref(false);
const playing = ref(false);
const muted = ref(true);
const video = ref<HTMLVideoElement | null>(null);

onMounted(() => {
  if (settings.value.autoPreview && props.resource.kind !== "image") {
    active.value = true;
  }
});

const startPreview = async () => {
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
  <div class="resource-media relative flex items-center justify-center rounded-t-[var(--ui-radius)]">
    <img
        v-if="resource.kind === 'image'"
        :src="resource.previewUrl || resource.preferredUrl"
        alt="Resource preview"
        loading="lazy"
        referrerpolicy="no-referrer"
        class="size-full object-contain"
    >

    <template v-else>
      <video
          v-if="active"
          ref="video"
          :src="resource.preferredUrl"
          :poster="resource.previewUrl"
          :muted="muted"
          playsinline
          preload="metadata"
          class="size-full object-contain"
          @play="playing = true"
          @pause="playing = false"
          @ended="playing = false"
      />
      <img
          v-else-if="resource.previewUrl"
          :src="resource.previewUrl"
          alt="Resource preview"
          loading="lazy"
          referrerpolicy="no-referrer"
          class="size-full object-contain"
      >
      <UIcon v-else name="i-lucide-film" class="size-12 text-muted"/>

      <div class="absolute inset-x-3 bottom-3 flex items-center justify-between gap-2">
        <UButton
            v-if="!active"
            label="Preview"
            icon="i-lucide-play"
            color="neutral"
            variant="solid"
            size="sm"
            @click="startPreview"
        />
        <template v-else>
          <UButton
              :icon="playing ? 'i-lucide-pause' : 'i-lucide-play'"
              color="neutral"
              variant="solid"
              size="sm"
              :aria-label="playing ? 'Pause preview' : 'Play preview'"
              @click="togglePlayback"
          />
          <UButton
              :icon="muted ? 'i-lucide-volume-x' : 'i-lucide-volume-2'"
              color="neutral"
              variant="solid"
              size="sm"
              :aria-label="muted ? 'Unmute preview' : 'Mute preview'"
              @click="toggleMuted"
          />
        </template>
      </div>
    </template>
  </div>
</template>
