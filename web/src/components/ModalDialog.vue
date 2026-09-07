<script setup lang="ts">
withDefaults(
  defineProps<{
    open: boolean;
    title: string;
    description?: string | undefined;
    wide?: boolean;
    closeOnBackdrop?: boolean;
  }>(),
  { closeOnBackdrop: true, description: undefined, wide: false },
);

const emit = defineEmits<{ close: [] }>();

function backdrop(event: MouseEvent) {
  if ((event.target as HTMLElement).dataset.backdrop === "true") emit("close");
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      data-backdrop="true"
      class="modal modal-open"
      role="presentation"
      @mousedown.self="closeOnBackdrop && backdrop($event)"
    >
      <section
        class="modal-box max-h-[90vh]"
        :class="wide ? 'max-w-4xl' : 'max-w-lg'"
        role="dialog"
        aria-modal="true"
        :aria-label="title"
      >
        <button
          class="btn btn-sm btn-circle btn-ghost absolute top-3 right-3"
          :aria-label="$t('common.close')"
          @click="emit('close')"
        >
          ✕
        </button>
        <h2 class="pr-10 text-xl font-bold">
          {{ title }}
        </h2>
        <p v-if="description" class="text-base-content/60 mt-1 text-sm">
          {{ description }}
        </p>
        <div class="mt-5">
          <slot />
        </div>
      </section>
      <button
        class="modal-backdrop"
        :aria-label="$t('common.close')"
        @click="emit('close')"
      >
        {{ $t("common.close") }}
      </button>
    </div>
  </Teleport>
</template>
