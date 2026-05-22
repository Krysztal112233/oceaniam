<script setup lang="ts">
import CloseIcon from "@iconify-vue/material-symbols/close-rounded";
import { ref, watch } from "vue";

const props = defineProps<{
    open: boolean;
    tenantId: string;
    loading: boolean;
    error: string | null;
}>();

const emit = defineEmits<{
    (event: "close"): void;
    (event: "submit"): void;
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

function handleSubmit() {
    emit("submit");
}
</script>

<template>
    <dialog ref="dialogRef" class="modal" @close="handleDialogClose">
        <div class="modal-box w-11/12 max-w-lg">
            <form method="dialog">
                <button
                    class="btn btn-circle btn-ghost btn-sm absolute right-2 top-2"
                    aria-label="Close"
                >
                    <CloseIcon class="h-4 w-4" />
                </button>
            </form>

            <h3 class="text-lg font-bold">新增 Secret</h3>
            <p class="mt-1 text-sm text-base-content/70">
                将在当前 tenant
                <span class="font-medium text-base-content">
                    {{ tenantId }}
                </span>
                上下文下创建一个新的 API Secret。
            </p>

            <div class="mt-4 rounded-box border border-base-200 bg-base-50 p-4">
                <p class="text-sm text-base-content/75">
                    创建完成后，页面会展示本次返回的完整 Secret
                    值。后续列表中仅显示脱敏内容。
                </p>
            </div>

            <form class="mt-4 space-y-4" @submit.prevent="handleSubmit">
                <div v-if="error" class="alert alert-error alert-soft">
                    <span>{{ error }}</span>
                </div>

                <button
                    type="submit"
                    class="btn btn-primary w-full"
                    :class="{ loading }"
                    :disabled="loading"
                >
                    创建 Secret
                </button>
            </form>
        </div>

        <form method="dialog" class="modal-backdrop">
            <button aria-label="Close"></button>
        </form>
    </dialog>
</template>
