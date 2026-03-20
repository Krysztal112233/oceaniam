<script setup lang="ts">
defineProps<{
    title: string;
    description?: string;
}>();
</script>

<template>
    <section class="rounded-box border border-base-200 bg-base-100">
        <div
            class="flex flex-col gap-6 px-5 py-5 lg:flex-row lg:items-start lg:justify-between"
        >
            <div class="lg:w-2/5 lg:shrink-0">
                <h3 class="text-base font-medium text-base-content">
                    {{ title }}
                </h3>
                <p v-if="description" class="mt-1 text-sm text-base-content/60">
                    {{ description }}
                </p>
            </div>

            <div class="hidden self-stretch lg:block">
                <div class="h-full w-1 bg-base-200"></div>
            </div>

            <div class="lg:w-3/5">
                <div
                    class="divide-y divide-base-200 rounded-box border border-base-200"
                >
                    <slot></slot>
                </div>
            </div>
        </div>
    </section>
</template>
