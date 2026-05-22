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
    (event: "submit", comment: string): void;
}>();

const dialogRef = ref<HTMLDialogElement | null>(null);
const comment = ref("");

watch(
    () => props.open,
    (open) => {
        const dialog = dialogRef.value;
        if (!dialog) return;

        if (open) {
            comment.value = "";
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
    emit("submit", comment.value);
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

            <h3 class="text-lg font-bold">新增 Application</h3>
            <p class="mt-1 text-sm text-base-content/70">
                将在当前 tenant
                <span class="font-medium text-base-content">
                    {{ tenantId }}
                </span>
                下创建一个新的 application。
            </p>

            <form class="mt-4 space-y-4" @submit.prevent="handleSubmit">
                <label class="form-control w-full">
                    <textarea
                        v-model="comment"
                        class="textarea textarea-bordered min-h-28 w-full mb-8"
                        placeholder="可选，输入 application 备注"
                    ></textarea>
                </label>

                <div v-if="error" class="alert alert-error alert-soft">
                    <span>{{ error }}</span>
                </div>

                <button
                    type="submit"
                    class="btn btn-primary w-full"
                    :class="{ loading }"
                    :disabled="loading"
                >
                    创建 Application
                </button>
            </form>
        </div>

        <form method="dialog" class="modal-backdrop">
            <button aria-label="Close"></button>
        </form>
    </dialog>
</template>
