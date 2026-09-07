<script setup lang="ts">
import { Icon } from "@iconify/vue/offline";

withDefaults(
  defineProps<{
    icon?: string;
    title: string;
    description?: string | undefined;
  }>(),
  {
    description: undefined,
    icon: "material-symbols:inbox-outline-rounded",
  },
);
</script>

<template>
  <div
    class="border-base-300 bg-base-100 flex flex-col items-center rounded-box border border-dashed px-6 py-14 text-center"
  >
    <div class="bg-base-200 mb-4 rounded-full p-4">
      <Icon :icon="icon" class="text-base-content/50 size-8" />
    </div>
    <h2 class="font-semibold">
      {{ title }}
    </h2>
    <p v-if="description" class="text-base-content/60 mt-1 max-w-lg text-sm">
      {{ description }}
    </p>
    <div v-if="$slots.default" class="mt-5">
      <slot />
    </div>
  </div>
</template>
