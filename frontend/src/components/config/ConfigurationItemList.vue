<script setup lang="ts">
import ConfigurationItemText from "./ConfigurationItemText.vue";

defineProps<{
    title: string;
    empty?: boolean;
    emptyText?: string;
}>();
</script>

<template>
    <div class="space-y-2 px-4 py-4">
        <p
            class="text-xs font-medium uppercase tracking-wide text-base-content/50"
        >
            {{ title }}
        </p>
        <div v-if="!empty" class="space-y-2">
            <slot></slot>
        </div>
        <ConfigurationItemText v-else :value="emptyText || '暂无配置'" muted />
    </div>
</template>
