<script setup lang="ts">
const props = defineProps<{
    tenantId: string;
}>();

const emit = defineEmits<{
    (event: "detail", tenantId: string): void;
    (event: "delete", tenantId: string): void;
}>();
</script>

<template>
    <div class="flex justify-end gap-2 whitespace-nowrap">
        <button
            type="button"
            class="btn btn-sm btn-ghost"
            @click="emit('detail', props.tenantId)"
        >
            详情
        </button>
        <button
            type="button"
            class="btn btn-sm btn-error btn-soft"
            @click="emit('delete', props.tenantId)"
        >
            删除
        </button>
    </div>
</template>
