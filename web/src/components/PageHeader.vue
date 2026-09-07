<script setup lang="ts">
defineProps<{ title: string; subtitle?: string }>();
</script>

<template>
  <header
    class="mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between"
  >
    <div>
      <h1 class="text-2xl font-bold tracking-tight sm:text-3xl">
        {{ title }}
      </h1>
      <p v-if="subtitle" class="text-base-content/60 mt-1">
        {{ subtitle }}
      </p>
    </div>
    <div
      v-if="$slots.actions"
      class="flex shrink-0 flex-wrap items-center gap-2"
    >
      <slot name="actions" />
    </div>
  </header>
</template>
