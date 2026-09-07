<script setup lang="ts">
import ModalDialog from "@/components/ModalDialog.vue";

withDefaults(
  defineProps<{
    open: boolean;
    title: string;
    message?: string | undefined;
    busy?: boolean;
    confirmLabel?: string | undefined;
  }>(),
  { busy: false, confirmLabel: undefined, message: undefined },
);

defineEmits<{ close: []; confirm: [] }>();
</script>

<template>
  <ModalDialog
    :open="open"
    :title="title"
    :close-on-backdrop="!busy"
    @close="$emit('close')"
  >
    <p>{{ message || $t("common.dangerConfirm") }}</p>
    <div class="modal-action">
      <button class="btn" :disabled="busy" @click="$emit('close')">
        {{ $t("common.cancel") }}
      </button>
      <button class="btn btn-error" :disabled="busy" @click="$emit('confirm')">
        <span v-if="busy" class="loading loading-spinner loading-sm" />
        {{ confirmLabel || $t("common.confirm") }}
      </button>
    </div>
  </ModalDialog>
</template>
