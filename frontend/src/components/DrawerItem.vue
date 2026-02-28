<script setup lang="ts">
import type { Component } from "vue";
import { RouterLink } from "vue-router";

defineProps<{
    tooltip: string;
    to: string;
    icon?: Component;
}>();
</script>

<template>
    <RouterLink
        :to="to"
        :title="tooltip"
        class="btn btn-ghost btn-sm w-full justify-start gap-2 lg:btn-md"
        active-class="btn-active"
    >
        <component
            v-if="icon"
            :is="icon"
            width="24"
            height="24"
            class="shrink-0"
        />
        <slot v-else name="icon"></slot>
        <span class="truncate">{{ tooltip }}</span>
    </RouterLink>
</template>
