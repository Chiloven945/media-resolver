<script setup lang="ts">
const props = defineProps<{ taskId: string }>();
const open = defineModel<boolean>("open", { default: false });
const value = ref("");
const busy = ref(false);
const queue = useTaskQueue();
const toast = useToast();

const pasteFromClipboard = async () => {
  try {
    const text = await navigator.clipboard.readText();
    if (!text.trim()) {
      toast.add({
        title: "Clipboard is empty",
        description: "Copy the response text first, then try again.",
        color: "neutral"
      });
      return;
    }
    value.value = text;
  } catch {
    toast.add({
      title: "Clipboard access unavailable",
      description: "Paste the response into the text area manually.",
      color: "neutral"
    });
  }
};

const submit = () => {
  if (!value.value.trim() || busy.value) {
    return;
  }

  busy.value = true;
  try {
    const result = queue.completeFromResponse(props.taskId, value.value);
    if (result.status === "ready") {
      toast.add({ title: "Response processed", color: "success" });
      value.value = "";
      open.value = false;
      return;
    }

    toast.add({
      title: "Response could not be used",
      description: "Copy the complete response text and try again.",
      color: "error"
    });
  } finally {
    busy.value = false;
  }
};

watch(open, isOpen => {
  if (!isOpen) {
    value.value = "";
  }
});
</script>

<template>
  <UModal
      v-model:open="open"
      title="Continue from response"
      description="Copy the complete response from the opened tab, then paste it here."
  >
    <template #body>
      <div class="space-y-3">
        <UAlert
            color="neutral"
            variant="subtle"
            icon="i-lucide-shield-check"
            title="Processed locally"
            description="The pasted response is passed directly to the local processing engine and is not stored in task history."
        />
        <UTextarea
            v-model="value"
            :rows="12"
            autoresize
            class="w-full"
            placeholder="Paste the complete response here"
            aria-label="Response text"
        />
      </div>
    </template>

    <template #footer>
      <div class="flex w-full flex-col-reverse gap-2 sm:flex-row sm:justify-between">
        <UButton
            label="Paste from clipboard"
            icon="i-lucide-clipboard-paste"
            color="neutral"
            variant="soft"
            @click="pasteFromClipboard"
        />
        <div class="flex justify-end gap-2">
          <UButton label="Cancel" color="neutral" variant="ghost" @click="open = false"/>
          <UButton
              label="Continue"
              icon="i-lucide-arrow-right"
              :loading="busy"
              :disabled="!value.trim()"
              @click="submit"
          />
        </div>
      </div>
    </template>
  </UModal>
</template>
