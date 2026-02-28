<template>
    <div class="drawer lg:drawer-open">
        <input :id="drawerId" type="checkbox" class="drawer-toggle" />

        <div class="drawer-content flex min-h-screen flex-col bg-base-100">
            <header class="navbar bg-base-100 lg:hidden">
                <div class="flex-none">
                    <label
                        :for="drawerId"
                        aria-label="Open sidebar"
                        class="btn btn-square btn-ghost"
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
                    </label>
                </div>
                <div class="flex-1 px-2 text-lg font-semibold">
                    {{ logoText }}
                </div>
            </header>

            <main class="flex-1 p-4">
                <slot name="content"></slot>
            </main>
        </div>

        <div class="drawer-side">
            <label
                :for="drawerId"
                aria-label="Close sidebar"
                class="drawer-overlay"
            ></label>

            <aside class="min-h-full w-64 bg-base-200 p-4 text-base-content">
                <div class="mb-4 flex items-center justify-start">
                    <span class="text-lg font-semibold">{{ logoText }}</span>
                </div>

                <nav class="flex flex-col gap-1">
                    <slot></slot>
                </nav>
            </aside>
        </div>
    </div>
</template>

<script setup lang="ts">
withDefaults(
    defineProps<{
        logoText: string;
        drawerId?: string;
    }>(),
    {
        drawerId: "app-drawer",
    },
);
</script>
