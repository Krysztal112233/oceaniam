<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(
    defineProps<{
        open?: boolean;
        drawerId?: string;
        topOffset?: number;
    }>(),
    {
        open: false,
        drawerId: "app-drawer",
        topOffset: 64,
    },
);

const emit = defineEmits<{
    (event: "update:open", value: boolean): void;
}>();

const drawerOpen = computed({
    get: () => props.open,
    set: (value) => emit("update:open", value),
});

const drawerId = computed(() => props.drawerId);
const topOffset = computed(() => props.topOffset);
</script>

<template>
    <div class="drawer" :class="{ 'lg:drawer-open': drawerOpen }">
        <input
            :id="drawerId"
            v-model="drawerOpen"
            type="checkbox"
            class="drawer-toggle"
        />

        <div class="drawer-content flex min-h-screen flex-col bg-base-100">
            <main class="flex-1 p-4" :style="{ paddingTop: `${topOffset}px` }">
                <slot name="content"></slot>
            </main>
        </div>

        <div class="drawer-side">
            <label
                :for="drawerId"
                aria-label="Close sidebar"
                class="drawer-overlay lg:hidden"
            ></label>

            <aside
                class="flex h-full min-h-full w-80 flex-col bg-base-200 p-4 text-base-content"
                :style="{ paddingTop: `${topOffset}px` }"
            >
                <nav class="flex flex-1 flex-col gap-1 overflow-y-auto">
                    <slot></slot>
                </nav>

                <div
                    v-if="$slots.footer"
                    class="mt-4 border-t border-base-300 pt-4"
                >
                    <slot name="footer"></slot>
                </div>
            </aside>
        </div>
    </div>
</template>
