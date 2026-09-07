<script setup lang="ts">
import { Icon } from "@iconify/vue/offline";

import { useToastsStore } from "@/stores/toasts";

const toasts = useToastsStore();
</script>

<template>
  <div class="toast toast-end toast-top z-[100]" aria-live="polite">
    <div
      v-for="item in toasts.messages"
      :key="item.id"
      class="alert max-w-sm shadow-lg"
      :class="{
        'alert-success': item.kind === 'success',
        'alert-error': item.kind === 'error',
        'alert-info': item.kind === 'info',
      }"
    >
      <Icon
        :icon="
          item.kind === 'success'
            ? 'material-symbols:check-circle-outline-rounded'
            : item.kind === 'error'
              ? 'material-symbols:error-outline-rounded'
              : 'material-symbols:info-outline-rounded'
        "
        class="size-5 shrink-0"
      />
      <span>{{ item.message }}</span>
      <button
        class="btn btn-ghost btn-xs btn-circle"
        :aria-label="$t('common.close')"
        @click="toasts.dismiss(item.id)"
      >
        <Icon icon="material-symbols:close-rounded" class="size-4" />
      </button>
    </div>
  </div>
</template>
