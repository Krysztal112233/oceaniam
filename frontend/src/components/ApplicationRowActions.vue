<script setup lang="ts">
import { ref, watch } from "vue";
import DeleteApplicationModal from "./DeleteApplicationModal.vue";

const props = defineProps<{
    applicationId: string;
    deleting?: boolean;
    deleteError?: string | null;
}>();

const emit = defineEmits<{
    (event: "detail", applicationId: string): void;
    (event: "delete", applicationId: string): void;
}>();

const isDialogOpen = ref(false);

watch(
    () => props.deleting,
    (deleting) => {
        if (!deleting && !props.deleteError && isDialogOpen.value) {
            isDialogOpen.value = false;
        }
    },
);

function openDeleteDialog() {
    isDialogOpen.value = true;
}

function closeDeleteDialog() {
    isDialogOpen.value = false;
}

function handleConfirmDelete() {
    emit("delete", props.applicationId);
}
</script>

<template>
    <div class="flex justify-center gap-2 whitespace-nowrap">
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
            :disabled="deleting"
            @click="openDeleteDialog"
        >
            删除
        </button>
    </div>

    <DeleteApplicationModal
        :open="isDialogOpen"
        :application-id="props.applicationId"
        :loading="Boolean(deleting)"
        :error="deleteError || null"
        @close="closeDeleteDialog"
        @confirm="handleConfirmDelete"
    />
</template>
