<script lang="ts" setup>
import type { DropdownMenuItem } from "@nuxt/ui";

const emit = defineEmits<{ openTasks: [] }>();
const preferencesOpen = ref(false);
const aboutOpen = ref(false);
const { settings } = useAppSettings();
const { state: engineState } = useEngine();
const config = useRuntimeConfig();

const concurrencyOptions = [1, 2, 3, 4, 5, 6, 7, 8];
const menuItems = computed<DropdownMenuItem[][]>(() => [
  [
    {
      label: "Preferences",
      icon: "i-lucide-settings-2",
      onSelect: () => {
        preferencesOpen.value = true;
      }
    },
    {
      label: "About",
      icon: "i-lucide-info",
      onSelect: () => {
        aboutOpen.value = true;
      }
    }
  ]
]);
</script>

<template>
  <UHeader
      :toggle="false"
      :ui="{ container: 'w-full max-w-none px-3 sm:px-4 lg:px-5' }"
      class="border-b border-default bg-default"
      data-testid="app-header"
  >
    <template #left>
      <UButton
          aria-label="Open tasks"
          class="lg:hidden"
          color="neutral"
          icon="i-lucide-list-todo"
          variant="ghost"
          @click="emit('openTasks')"
      />
      <div class="flex items-center gap-2.5">
        <UIcon class="size-6 text-primary" name="i-lucide-orbit"/>
        <span class="text-sm font-semibold text-highlighted sm:text-base">Media Resolver</span>
      </div>
    </template>

    <template #right>
      <UButton
          aria-label="New task"
          class="lg:hidden"
          color="primary"
          icon="i-lucide-plus"
          variant="soft"
          @click="emit('openTasks')"
      />
      <UBadge class="hidden sm:inline-flex" color="neutral" size="sm" variant="subtle">v0.1.0
      </UBadge>
      <UColorModeButton aria-label="Toggle color mode" color="neutral" variant="ghost"/>
      <UDropdownMenu :items="menuItems">
        <UButton
            aria-label="More options"
            color="neutral"
            icon="i-lucide-ellipsis"
            variant="ghost"
        />
      </UDropdownMenu>
    </template>
  </UHeader>

  <UModal
      v-model:open="preferencesOpen"
      description="These settings stay on this device."
      title="Preferences"
  >
    <template #body>
      <div class="space-y-5">
        <UFormField description="Choose how many remote requests can run at once."
                    label="Concurrent tasks">
          <USelect v-model="settings.concurrency" :items="concurrencyOptions" class="w-full"/>
        </UFormField>

        <div class="flex items-start justify-between gap-4">
          <div>
            <div class="text-sm font-medium text-highlighted">Auto-select new tasks</div>
            <div class="mt-1 text-sm text-muted">Move focus to a task when it is added.</div>
          </div>
          <USwitch v-model="settings.autoSelect"/>
        </div>

        <div class="flex items-start justify-between gap-4">
          <div>
            <div class="text-sm font-medium text-highlighted">Preview resources automatically</div>
            <div class="mt-1 text-sm text-muted">Prepare richer previews without starting
              playback.
            </div>
          </div>
          <USwitch v-model="settings.autoPreview"/>
        </div>
      </div>
    </template>
  </UModal>

  <UModal v-model:open="aboutOpen" description="Version 0.1.0" title="Media Resolver">
    <template #body>
      <div class="space-y-4 text-sm">
        <p class="text-muted">Resolve supported links into usable resources with a Rust-powered
          local processing engine.</p>
        <div class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-3">
          <span class="text-muted">Core</span>
          <span class="font-medium text-highlighted">0.1.0</span>
          <span class="text-muted">Engine</span>
          <span class="font-medium capitalize text-highlighted">{{ engineState }}</span>
          <span class="text-muted">Build</span>
          <span class="font-mono text-xs text-highlighted">{{ config.public.buildHash }}</span>
        </div>
      </div>
    </template>
  </UModal>
</template>
