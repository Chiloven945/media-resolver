<script setup lang="ts">
import type { DropdownMenuItem } from "@nuxt/ui";
import type { ResourceItem } from "~/types/engine";
import VariantList from "~/components/result/VariantList.vue";
import ResourcePreview from "~/components/result/ResourcePreview.vue";

const props = defineProps<{ resource: ResourceItem }>();
const variantsOpen = ref(false);
const toast = useToast();
const actions = useResourceActions();

const label = computed(() => (
    {
      image: "Image",
      video: "Video",
      animation: "Animation",
      unknown: "Resource"
    }[props.resource.kind]
));

const dimensions = computed(() => props.resource.width && props.resource.height
    ? `${props.resource.width}×${props.resource.height}`
    : "Original size");

const preferred = computed(() => props.resource.variants.find(variant => variant.url
        === props.resource.preferredUrl)
    || props.resource.variants[0]);
const bitrate = computed(() => preferred.value?.bitrate
    ? `${(
        preferred.value.bitrate / 1_000_000
    ).toFixed(1)} Mbps`
    : "");

const copy = async () => {
  await actions.copy(props.resource.preferredUrl);
  toast.add({
    title: "Copied",
    description: "Resource address copied to clipboard.",
    color: "success"
  });
};

const menuItems = computed<DropdownMenuItem[][]>(() => [
  [
    {
      label: "Open resource",
      icon: "i-lucide-external-link",
      onSelect: () => actions.open(props.resource.preferredUrl)
    },
    { label: "Copy address", icon: "i-lucide-copy", onSelect: copy },
    ...(
        props.resource.variants.length > 1
            ? [
              {
                label: "View variants", icon: "i-lucide-list-tree", onSelect: () => {
                  variantsOpen.value = true;
                }
              }
            ]
            : []
    )
  ]
]);
</script>

<template>
  <UCard :ui="{ body: 'p-0 sm:p-0', footer: 'p-4 sm:px-4' }" class="overflow-hidden">
    <ResourcePreview :resource="resource"/>

    <div class="space-y-4 p-4">
      <div class="flex items-start justify-between gap-3">
        <div>
          <div class="flex items-center gap-2">
            <UBadge color="neutral" variant="subtle">{{ label }}</UBadge>
            <span class="text-sm font-medium text-highlighted">{{ dimensions }}</span>
          </div>
          <p v-if="bitrate" class="mt-2 text-xs text-muted">{{ bitrate }}</p>
        </div>
        <UBadge v-if="resource.variants.length > 1" color="primary" variant="subtle">
          {{ resource.variants.length }} variants
        </UBadge>
      </div>
    </div>

    <template #footer>
      <div class="flex items-center gap-2">
        <UButton
            label="Open"
            icon="i-lucide-external-link"
            color="neutral"
            variant="outline"
            class="flex-1"
            @click="actions.open(resource.preferredUrl)"
        />
        <UButton
            label="Copy"
            icon="i-lucide-copy"
            class="flex-1"
            @click="copy"
        />
        <UDropdownMenu :items="menuItems">
          <UButton icon="i-lucide-ellipsis"
                   color="neutral"
                   variant="ghost"
                   aria-label="Resource actions"/>
        </UDropdownMenu>
      </div>
    </template>
  </UCard>

  <UModal v-model:open="variantsOpen" title="Resource variants"
          description="Choose an available representation of this resource.">
    <template #body>
      <VariantList :variants="resource.variants"/>
    </template>
  </UModal>
</template>
