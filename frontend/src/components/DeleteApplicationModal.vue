<script setup lang="ts">
import { ref, watch } from "vue";

const props = defineProps<{
    open: boolean;
    applicationId: string;
    loading: boolean;
    error: string | null;
}>();

const emit = defineEmits<{
    (event: "close"): void;
    (event: "confirm", applicationId: string): void;
}>();

const dialogRef = ref<HTMLDialogElement | null>(null);

watch(
    () => props.open,
    (open) => {
        const dialog = dialogRef.value;
        if (!dialog) return;

        if (open) {
            try {
                if (!dialog.open) dialog.showModal();
            } catch {
                // ignore
            }
            return;
        }

        if (dialog.open) dialog.close();
    },
    {
        immediate: true,
    },
);

function handleDialogClose() {
    if (props.open) emit("close");
}

function handleConfirm() {
    emit("confirm", props.applicationId);
}
</script>

<template>
    <dialog ref="dialogRef" class="modal" @close="handleDialogClose">
        <div class="modal-box w-11/12 max-w-md">
            <h3 class="text-lg font-bold">确认删除 Application</h3>
            <p class="mt-2 text-sm text-base-content/70">
                即将删除 application
                <span class="font-mono text-base-content">
                    {{ applicationId }}
                </span>
                。删除后不可恢复，请再次确认。
            </p>

            <div v-if="error" class="alert alert-error alert-soft mt-4">
                <span>{{ error }}</span>
            </div>

            <div class="modal-action">
                <form method="dialog">
                    <button
                        type="submit"
                        class="btn btn-ghost"
                        :disabled="loading"
                    >
                        取消
                    </button>
                </form>
                <button
                    type="button"
                    class="btn btn-error"
                    :class="{ loading }"
                    :disabled="loading"
                    @click="handleConfirm"
                >
                    确认删除
                </button>
            </div>
        </div>

        <form method="dialog" class="modal-backdrop">
            <button aria-label="Close" :disabled="loading"></button>
        </form>
    </dialog>
</template>
