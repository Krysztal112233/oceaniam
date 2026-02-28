<script setup lang="ts">
import { RouterLink } from "vue-router";

const props = withDefaults(
    defineProps<{
        brandText: string;
        drawerOpen?: boolean;
        drawerId?: string;
    }>(),
    {
        drawerOpen: false,
        drawerId: "app-drawer",
    },
);

const emit = defineEmits<{
    (event: "toggle-drawer"): void;
}>();
</script>

<template>
    <header
        class="navbar fixed top-0 z-50 h-16 w-full border-b border-base-200 bg-base-100/80 px-2 backdrop-blur shadow-sm"
    >
        <div class="flex-none">
            <button
                type="button"
                aria-label="Toggle sidebar"
                class="btn btn-square btn-ghost"
                :aria-controls="props.drawerId"
                :aria-expanded="props.drawerOpen"
                @click="emit('toggle-drawer')"
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                    class="h-6 w-6 stroke-current"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M4 6h16M4 12h16M4 18h16"
                    />
                </svg>
            </button>
        </div>

        <div class="flex-1 min-w-0">
            <RouterLink
                to="/"
                class="btn btn-ghost text-lg font-semibold normal-case"
            >
                {{ props.brandText }}
            </RouterLink>
        </div>

        <div class="flex-none">
            <slot name="right"></slot>
        </div>
    </header>
</template>
