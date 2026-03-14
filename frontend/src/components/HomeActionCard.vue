<script setup lang="ts">
import type { Component } from "vue";

defineProps<{
    to: string;
    title: string;
    description: string;
    ctaLabel: string;
    icon: Component;
    accent: "primary" | "secondary";
}>();
</script>

<template>
    <RouterLink
        :to="to"
        class="card border border-base-200 bg-base-100 shadow-sm transition hover:-translate-y-0.5 hover:shadow-md"
        :class="
            accent === 'primary'
                ? 'hover:border-primary/40'
                : 'hover:border-secondary/40'
        "
    >
        <div class="card-body gap-4">
            <div
                class="flex h-12 w-12 items-center justify-center rounded-box"
                :class="
                    accent === 'primary'
                        ? 'bg-primary/10 text-primary'
                        : 'bg-secondary/10 text-secondary'
                "
            >
                <component :is="icon" class="h-6 w-6" />
            </div>

            <div class="space-y-2">
                <h2 class="text-lg font-semibold text-base-content">
                    {{ title }}
                </h2>
                <p class="text-sm leading-6 text-base-content/70">
                    {{ description }}
                </p>
            </div>

            <div class="card-actions justify-start">
                <span
                    class="btn btn-sm"
                    :class="
                        accent === 'primary'
                            ? 'btn-primary'
                            : 'btn-secondary'
                    "
                >
                    {{ ctaLabel }}
                </span>
            </div>
        </div>
    </RouterLink>
</template>
