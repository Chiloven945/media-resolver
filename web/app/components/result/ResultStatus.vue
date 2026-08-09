<script lang="ts" setup>
import type { PublicErrorMessage, ResolveTask } from "~/types/task";

const props = defineProps<{
  task: ResolveTask;
  errorMessage: PublicErrorMessage;
  recoveryAvailable: boolean;
}>();
const emit = defineEmits<{ retry: []; cancel: []; recover: [] }>();

const detail = computed(() => {
  switch (props.task.state) {
    case "queued":
      return {
        icon: "i-lucide-clock-3",
        title: "Waiting in queue",
        description: "This task will start as soon as a processing slot is available.",
        tone: "text-muted"
      };
    case "connecting":
      return {
        icon: "i-lucide-loader-circle",
        title: "Connecting to source",
        description: "The task will continue automatically when a response is available.",
        tone: "text-primary"
      };
    case "processing":
      return {
        icon: "i-lucide-loader-circle",
        title: "Processing response",
        description: "The local engine is preparing usable resources.",
        tone: "text-primary"
      };
    case "failed":
      return {
        icon: "i-lucide-circle-alert",
        title: props.errorMessage.title,
        description: props.errorMessage.description,
        tone: "text-error"
      };
    default:
      return {
        icon: "i-lucide-ban",
        title: "Task cancelled",
        description: "You can retry this task whenever you are ready.",
        tone: "text-muted"
      };
  }
});

const active = computed(() => props.task.state
    === "connecting"
    || props.task.state
    === "processing");
</script>

<template>
  <div class="flex min-h-full items-center justify-center p-5 sm:p-8">
    <div class="w-full max-w-xl border-y border-default py-8 text-center sm:py-10">
      <UIcon
          :class="[detail.tone, { 'animate-spin': active }]"
          :name="detail.icon"
          class="mx-auto size-7"
      />
      <h2 class="mt-4 text-base font-semibold text-highlighted">{{ detail.title }}</h2>
      <p class="mx-auto mt-2 max-w-md text-sm leading-6 text-muted">{{ detail.description }}</p>

      <UProgress v-if="active" animation="carousel" class="mx-auto mt-6 max-w-sm" size="xs"/>

      <div class="mt-6 flex flex-wrap justify-center gap-2">
        <UButton
            v-if="active"
            color="neutral"
            icon="i-lucide-circle-stop"
            label="Cancel"
            variant="outline"
            @click="emit('cancel')"
        />
        <template v-else-if="task.state === 'failed'">
          <UButton icon="i-lucide-refresh-cw" label="Retry" @click="emit('retry')"/>
          <UButton
              v-if="recoveryAvailable"
              color="neutral"
              icon="i-lucide-life-buoy"
              label="Advanced recovery"
              variant="outline"
              @click="emit('recover')"
          />
        </template>
        <UButton
            v-else-if="task.state === 'cancelled'"
            color="neutral"
            icon="i-lucide-refresh-cw"
            label="Retry"
            variant="outline"
            @click="emit('retry')"
        />
      </div>
    </div>
  </div>
</template>
