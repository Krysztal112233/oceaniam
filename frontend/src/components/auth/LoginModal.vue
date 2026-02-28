<script setup lang="ts">
import { ref, watch } from "vue";
import { authState, login } from "../../auth";

const props = defineProps<{
    open: boolean;
}>();

const emit = defineEmits<{
    (event: "close"): void;
    (event: "success"): void;
}>();

const dialogRef = ref<HTMLDialogElement | null>(null);
const username = ref("");
const password = ref("");

watch(
    () => props.open,
    (open) => {
        const dialog = dialogRef.value;
        if (!dialog) return;

        authState.error = null;

        if (open) {
            password.value = "";
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

async function handleSubmit() {
    try {
        await login(username.value, password.value);
        emit("success");
    } catch {
        // error is stored in authState.error
    }
}
</script>

<template>
    <dialog ref="dialogRef" class="modal" @close="handleDialogClose">
        <div class="modal-box w-11/12 max-w-md">
            <form method="dialog">
                <button
                    class="btn btn-circle btn-ghost btn-sm absolute right-2 top-2"
                    aria-label="Close"
                >
                    ✕
                </button>
            </form>

            <h3 class="text-lg font-bold">登录</h3>
            <p class="mt-1 text-sm opacity-70">
                登录以继续使用 OceanIAM 管理台
            </p>

            <form
                class="mt-4 flex flex-col gap-3"
                @submit.prevent="handleSubmit"
            >
                <label class="form-control w-full">
                    <div class="label">
                        <span class="label-text">账号</span>
                    </div>
                    <input
                        v-model="username"
                        type="text"
                        autocomplete="username"
                        class="input input-bordered w-full"
                        placeholder="请输入账号"
                    />
                </label>

                <label class="form-control w-full">
                    <div class="label">
                        <span class="label-text">密码</span>
                    </div>
                    <input
                        v-model="password"
                        type="password"
                        autocomplete="current-password"
                        class="input input-bordered w-full"
                        placeholder="请输入密码"
                    />
                </label>

                <div
                    v-if="authState.error"
                    class="alert alert-error py-2 text-sm"
                >
                    <span>{{ authState.error }}</span>
                </div>

                <button
                    type="submit"
                    class="btn btn-primary w-full"
                    :class="{ loading: authState.loading }"
                    :disabled="authState.loading"
                >
                    登录
                </button>
            </form>
        </div>

        <form method="dialog" class="modal-backdrop">
            <button aria-label="Close"></button>
        </form>
    </dialog>
</template>
