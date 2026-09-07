<script setup lang="ts">
withDefaults(
  defineProps<{
    page: number;
    total: number;
    hasNext: boolean;
    perPage?: number;
  }>(),
  { perPage: 20 },
);

defineEmits<{ change: [page: number] }>();
</script>

<template>
  <div class="mt-5 flex flex-wrap items-center justify-between gap-3">
    <span class="text-base-content/60 text-sm">{{
      $t("common.total", { total })
    }}</span>
    <div class="join">
      <button
        class="btn join-item btn-sm"
        :disabled="page <= 1"
        @click="$emit('change', page - 1)"
      >
        {{ $t("common.previous") }}
      </button>
      <button class="btn join-item btn-sm pointer-events-none">
        {{ $t("common.page", { page }) }}
      </button>
      <button
        class="btn join-item btn-sm"
        :disabled="!hasNext"
        @click="$emit('change', page + 1)"
      >
        {{ $t("common.next") }}
      </button>
    </div>
  </div>
</template>
