<script setup lang="ts">
import CloseIcon from "@iconify-vue/material-symbols/close-rounded";
import { ref, watch } from "vue";

const props = defineProps<{
    open: boolean;
    tenantId: string;
    applicationId: string;
    userId: string;
    nickname: string;
    loading: boolean;
    error: string | null;
}>();

const emit = defineEmits<{
    (event: "close"): void;
    (event: "submit", password: string): void;
}>();

const dialogRef = ref<HTMLDialogElement | null>(null);
const newPassword = ref("");
const confirmPassword = ref("");
const localValidationError = ref<string | null>(null);

watch(
    () => props.open,
    (open) => {
        const dialog = dialogRef.value;
        if (!dialog) return;

        if (open) {
            newPassword.value = "";
            confirmPassword.value = "";
            localValidationError.value = null;
            try {
                if (!dialog.open) dialog.showModal();
            } catch {
                // ignore
            }
            return;
        }

        if (dialog.open) dialog.close();
    },
    { immediate: true },
);

function handleDialogClose(): void {
    if (props.open) emit("close");
}

function handleSubmit(): void {
    const password = newPassword.value;
    const confirm = confirmPassword.value;

    if (password.length < 12) {
        localValidationError.value = "密码长度至少 12 位。";
        return;
    }

    if (password !== confirm) {
        localValidationError.value = "两次输入密码不一致。";
        return;
    }

    localValidationError.value = null;
    emit("submit", password);
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

            <h3 class="text-lg font-bold">修改密码</h3>
            <p class="mt-1 text-sm text-base-content/70">
                为用户
                <span class="font-medium text-base-content">
                    {{ nickname }}
                </span>
                设置新密码。
            </p>

            <form class="mt-4 space-y-4" @submit.prevent="handleSubmit">
                <label class="form-control w-full">
                    <div class="label">
                        <span class="label-text">新密码</span>
                    </div>
                    <input
                        v-model="newPassword"
                        type="password"
                        class="input input-bordered w-full"
                        placeholder="至少 12 位"
                        :disabled="loading"
                    />
                </label>

                <label class="form-control w-full">
                    <div class="label">
                        <span class="label-text">确认密码</span>
                    </div>
                    <input
                        v-model="confirmPassword"
                        type="password"
                        class="input input-bordered w-full"
                        placeholder="再次输入新密码"
                        :disabled="loading"
                    />
                </label>

                <div
                    v-if="localValidationError || error"
                    class="alert alert-error alert-soft"
                >
                    <span>{{ localValidationError || error }}</span>
                </div>

                <button
                    type="submit"
                    class="btn btn-primary w-full"
                    :class="{ loading }"
                    :disabled="loading"
                >
                    保存密码
                </button>
            </form>
        </div>

        <form method="dialog" class="modal-backdrop">
            <button aria-label="Close"></button>
        </form>
    </dialog>
</template>
