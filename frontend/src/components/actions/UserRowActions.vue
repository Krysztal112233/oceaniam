<script setup lang="ts">
import { ref, watch } from "vue";
import DeleteApplicationUserModal from "../modals/DeleteApplicationUserModal.vue";

const props = defineProps<{
    userId: string;
    nickname: string;
    deleting?: boolean;
    deleteError?: string | null;
}>();

const emit = defineEmits<{
    (event: "changePassword"): void;
    (event: "delete", userId: string): void;
    (event: "deleteClose"): void;
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
    emit("deleteClose");
}

function handleConfirmDelete() {
    emit("delete", props.userId);
}
</script>

<template>
    <div class="flex justify-center gap-2 whitespace-nowrap">
        <button
            type="button"
            class="btn btn-sm btn-ghost"
            @click="emit('changePassword')"
        >
            修改密码
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

    <DeleteApplicationUserModal
        :open="isDialogOpen"
        :user-id="props.userId"
        :nickname="props.nickname"
        :loading="Boolean(deleting)"
        :error="deleteError || null"
        @close="closeDeleteDialog"
        @confirm="handleConfirmDelete"
    />
</template>
