<script setup lang="ts">
import type { Component } from "vue";

withDefaults(
    defineProps<{
        title: string;
        value: string | number;
        desc?: string;
        icon?: Component;
        figureClass?: string;
        iconSize?: number;
    }>(),
    {
        figureClass: "",
        iconSize: 32,
    },
);
</script>

<template>
    <div class="stat">
        <div v-if="icon || $slots.icon" :class="['stat-figure', figureClass]">
            <component
                v-if="icon"
                :is="icon"
                :width="iconSize"
                :height="iconSize"
            />
            <slot v-else name="icon"></slot>
        </div>

        <div class="stat-title">{{ title }}</div>
        <div class="stat-value">{{ value }}</div>
        <div v-if="desc" class="stat-desc">{{ desc }}</div>
    </div>
</template>

