<script setup lang="ts">
import { Icon } from "@iconify/vue/offline";

defineProps<{ message?: string | undefined }>();
defineEmits<{ retry: [] }>();
</script>

<template>
  <div role="alert" class="alert alert-error alert-soft">
    <Icon icon="material-symbols:error-outline-rounded" class="size-5" />
    <span>{{ message || $t("common.requestFailed") }}</span>
    <button class="btn btn-sm" @click="$emit('retry')">
      {{ $t("common.retry") }}
    </button>
  </div>
</template>
