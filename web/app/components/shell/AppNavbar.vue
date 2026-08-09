<script setup lang="ts">
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
  <UDashboardNavbar>
    <template #left>
      <UButton
          icon="i-lucide-list-todo"
          color="neutral"
          variant="ghost"
          class="lg:hidden"
          aria-label="Open tasks"
          @click="emit('openTasks')"
      />
      <div class="flex items-center gap-3">
        <div
            class="flex size-9 items-center justify-center rounded-xl bg-primary/10 text-primary ring-1 ring-primary/20">
          <UIcon name="i-lucide-orbit" class="size-5"/>
        </div>
        <div class="min-w-0">
          <div class="truncate text-sm font-semibold text-highlighted">Media Resolver</div>
          <div class="hidden text-xs text-muted sm:block">Resolve supported links into usable
            resources.
          </div>
        </div>
      </div>
    </template>

    <template #right>
      <UButton
          icon="i-lucide-plus"
          color="primary"
          variant="soft"
          class="lg:hidden"
          aria-label="New task"
          @click="emit('openTasks')"
      />
      <UBadge color="neutral" variant="subtle" class="hidden sm:inline-flex">v0.1.0</UBadge>
      <UColorModeButton color="neutral" variant="ghost"/>
      <UDropdownMenu :items="menuItems">
        <UButton icon="i-lucide-ellipsis"
                 color="neutral"
                 variant="ghost"
                 aria-label="More options"/>
      </UDropdownMenu>
    </template>
  </UDashboardNavbar>

  <UModal v-model:open="preferencesOpen"
          title="Preferences"
          description="These settings stay on this device.">
    <template #body>
      <div class="space-y-6">
        <UFormField label="Concurrent tasks"
                    description="Choose how many remote requests can run at once.">
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

  <UModal v-model:open="aboutOpen" title="Media Resolver" description="Version 0.1.0">
    <template #body>
      <div class="space-y-4 text-sm">
        <p class="text-muted">Rust-powered local processing engine with a Nuxt UI client
          interface.</p>
        <div class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-3">
          <span class="text-muted">Core</span>
          <span class="font-medium text-highlighted">0.1.0</span>
          <span class="text-muted">Engine</span>
          <span><UBadge :color="engineState === 'ready' ? 'success' : engineState === 'error' ? 'error' : 'neutral'"
                        variant="subtle">{{ engineState }}</UBadge></span>
          <span class="text-muted">Build</span>
          <span class="font-mono text-xs text-highlighted">{{ config.public.buildHash }}</span>
        </div>
      </div>
    </template>
  </UModal>
</template>
