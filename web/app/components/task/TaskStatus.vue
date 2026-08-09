<script setup lang="ts">
import type { TaskState } from "~/types/task";

const props = defineProps<{ state: TaskState }>();

type StatusDetail = {
  label: string
  color: "neutral" | "info" | "primary" | "success" | "error"
  icon: string
  loading: boolean
}

const statusDetails: Record<TaskState, StatusDetail> = {
  queued: {
    label: "Queued",
    color: "neutral",
    icon: "i-lucide-clock-3",
    loading: false
  },
  connecting: {
    label: "Connecting",
    color: "info",
    icon: "i-lucide-loader-circle",
    loading: true
  },
  processing: {
    label: "Processing",
    color: "primary",
    icon: "i-lucide-loader-circle",
    loading: true
  },
  ready: {
    label: "Ready",
    color: "success",
    icon: "i-lucide-circle-check",
    loading: false
  },
  failed: {
    label: "Failed",
    color: "error",
    icon: "i-lucide-circle-x",
    loading: false
  },
  cancelled: {
    label: "Cancelled",
    color: "neutral",
    icon: "i-lucide-ban",
    loading: false
  }
};

const detail = computed(() => statusDetails[props.state]);
</script>

<template>
  <UBadge :color="detail.color" variant="subtle" size="sm">
    <UIcon :name="detail.icon" class="size-3.5" :class="{ 'animate-spin': detail.loading }"/>
    {{ detail.label }}
  </UBadge>
</template>
