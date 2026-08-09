<script setup lang="ts">
const emit = defineEmits<{ newTask: [] }>();
</script>

<template>
  <div class="flex min-h-[calc(100dvh-var(--ui-header-height))] items-center justify-center p-6">
    <UEmpty
        icon="i-lucide-orbit"
        title="No active tasks"
        description="Add a supported link to start. Processing happens locally when possible."
        variant="subtle"
        class="w-full max-w-xl"
    >
      <template #actions>
        <UButton label="New task" icon="i-lucide-plus" @click="emit('newTask')"/>
      </template>
    </UEmpty>
  </div>
</template>
