<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";
import Drawer from "../components/Drawer.vue";
import AppNavbar from "./AppNavbar.vue";

const props = withDefaults(
    defineProps<{
        brandText: string;
        drawerId?: string;
        navbarHeight?: number;
    }>(),
    {
        drawerId: "app-drawer",
        navbarHeight: 64,
    },
);

const drawerOpen = ref(false);

function toggleDrawer() {
    drawerOpen.value = !drawerOpen.value;
}

let mediaQueryList: MediaQueryList | null = null;
let mediaQueryListener: ((ev: MediaQueryListEvent) => void) | null = null;

function syncDrawerDefaultToViewport() {
    if (!mediaQueryList) return;
    drawerOpen.value = mediaQueryList.matches;
}

onMounted(() => {
    if (typeof window === "undefined" || !window.matchMedia) return;

    mediaQueryList = window.matchMedia("(min-width: 1024px)");
    syncDrawerDefaultToViewport();

    mediaQueryListener = () => {
        syncDrawerDefaultToViewport();
    };

    if (mediaQueryList.addEventListener) {
        mediaQueryList.addEventListener("change", mediaQueryListener);
        return;
    }

    mediaQueryList.addListener(mediaQueryListener);
});

onBeforeUnmount(() => {
    if (!mediaQueryList || !mediaQueryListener) return;

    if (mediaQueryList.removeEventListener) {
        mediaQueryList.removeEventListener("change", mediaQueryListener);
        return;
    }

    mediaQueryList.removeListener(mediaQueryListener);
});
</script>

<template>
    <AppNavbar
        :brand-text="props.brandText"
        :drawer-open="drawerOpen"
        :drawer-id="props.drawerId"
        @toggle-drawer="toggleDrawer"
    >
        <template #right>
            <slot name="navbar-right"></slot>
        </template>
    </AppNavbar>

    <Drawer
        v-model:open="drawerOpen"
        :drawer-id="props.drawerId"
        :top-offset="props.navbarHeight"
    >
        <slot></slot>

        <template #content>
            <slot name="content"></slot>
        </template>

        <template v-if="$slots.footer" #footer>
            <slot name="footer"></slot>
        </template>
    </Drawer>
</template>
