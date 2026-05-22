<script setup lang="ts">
defineProps<{
    pageTitle: string;
    pageDescription: string;
    cardTitle?: string;
    summaryText?: string;
}>();
</script>

<template>
    <section class="flex flex-col gap-6">
        <header class="flex flex-col gap-2">
            <h1 class="text-2xl font-semibold text-base-content">
                {{ pageTitle }}
            </h1>
            <p class="text-sm text-base-content/70">
                {{ pageDescription }}
            </p>
        </header>

        <div class="card border border-base-200 bg-base-100 shadow-sm">
            <div class="card-body gap-4 p-0">
                <div
                    class="flex items-center justify-between border-b border-base-200 px-6 py-4"
                >
                    <div class="space-y-1">
                        <h2
                            v-if="cardTitle"
                            class="text-lg font-medium text-base-content"
                        >
                            {{ cardTitle }}
                        </h2>
                        <p
                            v-if="summaryText"
                            class="text-sm text-base-content/60"
                        >
                            {{ summaryText }}
                        </p>
                    </div>

                    <slot name="actions"></slot>
                </div>

                <slot></slot>
            </div>
        </div>
    </section>
</template>
