<script setup lang="ts">
const props = defineProps<{
    title?: string;
    value: string;
    muted?: boolean;
}>();
</script>

<template>
    <div
        class="space-y-2"
        :class="props.title ? 'px-4 py-4' : ''"
    >
        <p
            v-if="props.title"
            class="text-xs font-medium uppercase tracking-wide text-base-content/50"
        >
            {{ props.title }}
        </p>
        <div class="rounded-box border border-base-300 bg-base-50 px-3 py-2">
            <p
                class="break-all text-sm"
                :class="
                    props.muted ? 'text-base-content/50' : 'text-base-content'
                "
            >
                {{ props.value }}
            </p>
        </div>
    </div>
</template>
