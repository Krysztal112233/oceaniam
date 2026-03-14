<script setup lang="ts">
import ExpandMoreIcon from "@iconify-vue/material-symbols/expand-more";
import { computed, ref, watch } from "vue";
import { useRoute } from "vue-router";
import DrawerItem from "./DrawerItem.vue";

type DrawerGroupItem = {
    label: string;
    to: string;
};

const props = defineProps<{
    label: string;
    items: DrawerGroupItem[];
}>();

const route = useRoute();

const hasActiveChild = computed(() =>
    props.items.some((item) => route.path === item.to),
);

const expanded = ref(hasActiveChild.value);

watch(hasActiveChild, (value) => {
    if (value) {
        expanded.value = true;
    }
});
</script>

<template>
    <div class="flex flex-col gap-1">
        <button
            type="button"
            class="btn btn-ghost btn-sm w-full justify-start gap-2 lg:btn-md"
            :class="{ 'btn-active': hasActiveChild }"
            :aria-expanded="expanded"
            @click="expanded = !expanded"
        >
            <span class="min-w-0 flex-1 truncate text-left">{{
                props.label
            }}</span>
            <ExpandMoreIcon
                width="24"
                height="24"
                class="shrink-0 transition-transform"
                :class="{ 'rotate-180': expanded }"
            />
        </button>

        <div
            v-show="expanded"
            class="ml-4 flex flex-col gap-1 border-l border-base-300 pl-2"
        >
            <DrawerItem
                v-for="item in props.items"
                :key="item.to"
                :tooltip="item.label"
                :to="item.to"
            />
        </div>
    </div>
</template>
