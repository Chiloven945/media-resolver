<script lang="ts" setup>
const props = defineProps<{ taskId: string }>();
const open = defineModel<boolean>("open", { default: false });
const value = ref("");
const busy = ref(false);
const queue = useTaskQueue();
const toast = useToast();

const openResponse = () => {
  if (!queue.openResponse(props.taskId)) {
    toast.add({
      title: "Unable to open response",
      description: "Allow pop-ups for this page and try again.",
      color: "error"
    });
    return;
  }
  toast.add({
    title: "Response opened",
    description: "Return here with the complete response text to continue.",
    color: "neutral"
  });
};

const pasteFromClipboard = async () => {
  try {
    const text = await navigator.clipboard.readText();
    if (!text.trim()) {
      toast.add({
        title: "Clipboard is empty",
        description: "Place the response text on the clipboard first, then try again.",
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

const submit = async () => {
  if (!value.value.trim() || busy.value) {
    return;
  }

  busy.value = true;
  try {
    const result = await queue.completeFromResponse(props.taskId, value.value);
    if (result.status === "ready") {
      toast.add({ title: "Response processed", color: "success" });
      value.value = "";
      open.value = false;
      return;
    }

    toast.add({
      title: "Response could not be used",
      description: "Use the complete response text and try again.",
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
      description="Use a response that your browser can open but could not read automatically."
      title="Advanced recovery"
  >
    <template #body>
      <div class="space-y-3">
        <UAlert
            color="neutral"
            description="The pasted response is passed directly to the local processing engine and is not stored in task history."
            icon="i-lucide-shield-check"
            title="Processed locally"
            variant="subtle"
        >
          <template #actions>
            <UButton
                color="neutral"
                icon="i-lucide-external-link"
                label="Open response"
                variant="soft"
                @click="openResponse"
            />
          </template>
        </UAlert>
        <UTextarea
            v-model="value"
            :rows="12"
            aria-label="Response text"
            autoresize
            class="w-full"
            placeholder="Paste the complete response here"
        />
      </div>
    </template>

    <template #footer>
      <div class="flex w-full flex-col-reverse gap-2 sm:flex-row sm:justify-between">
        <UButton
            color="neutral"
            icon="i-lucide-clipboard-paste"
            label="Paste from clipboard"
            variant="soft"
            @click="pasteFromClipboard"
        />
        <div class="flex justify-end gap-2">
          <UButton color="neutral" label="Cancel" variant="ghost" @click="open = false"/>
          <UButton
              :disabled="!value.trim()"
              :loading="busy"
              icon="i-lucide-arrow-right"
              label="Continue"
              @click="submit"
          />
        </div>
      </div>
    </template>
  </UModal>
</template>
