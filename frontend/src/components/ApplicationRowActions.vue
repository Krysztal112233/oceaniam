<script setup lang="ts">
const props = defineProps<{
    applicationId: string;
}>();

const emit = defineEmits<{
    (event: "detail", applicationId: string): void;
    (event: "delete", applicationId: string): void;
}>();
</script>

<template>
    <div class="flex justify-end gap-2 whitespace-nowrap">
        <button
            type="button"
            class="btn btn-sm btn-ghost"
            @click="emit('detail', props.applicationId)"
        >
            详情
        </button>
        <button
            type="button"
            class="btn btn-sm btn-error btn-soft"
            @click="emit('delete', props.applicationId)"
        >
            删除
        </button>
    </div>
</template>
