<script setup lang="ts">
import CasinoIcon from "@iconify-vue/material-symbols/casino-outline";
import CloseIcon from "@iconify-vue/material-symbols/close-rounded";
import { computed, ref, watch } from "vue";
import {
    adjectives,
    animals,
    uniqueNamesGenerator,
} from "unique-names-generator";

type ApplicationOption = {
    id: string;
    label: string;
};

type CreateUserPayload = {
    applicationId: string;
    nickname: string;
    password: string;
    email: string | null;
    phone: string | null;
};

const props = defineProps<{
    open: boolean;
    tenantId: string;
    loading: boolean;
    error: string | null;
    applications: ApplicationOption[];
}>();

const emit = defineEmits<{
    (event: "close"): void;
    (event: "submit", payload: CreateUserPayload): void;
}>();

const dialogRef = ref<HTMLDialogElement | null>(null);
const selectedApplicationId = ref("");
const nickname = ref("");
const password = ref("");
const email = ref("");
const phone = ref("");
const localValidationError = ref<string | null>(null);

const hasApplicationOptions = computed(() => props.applications.length > 0);

function generateNickname(): string {
    return uniqueNamesGenerator({
        dictionaries: [adjectives, animals],
        separator: "-",
        style: "lowerCase",
    });
}

watch(
    () => props.open,
    (open) => {
        const dialog = dialogRef.value;
        if (!dialog) return;

        if (open) {
            selectedApplicationId.value = props.applications[0]?.id ?? "";
            nickname.value = generateNickname();
            password.value = "";
            email.value = "";
            phone.value = "";
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
    {
        immediate: true,
    },
);

watch(
    () => props.applications,
    (applications) => {
        if (!props.open) {
            return;
        }

        const matchedApplication = applications.find(
            (application) => application.id === selectedApplicationId.value,
        );
        if (!matchedApplication) {
            selectedApplicationId.value = applications[0]?.id ?? "";
        }
    },
    { deep: true },
);

function handleDialogClose(): void {
    if (props.open) emit("close");
}

function handleSubmit(): void {
    const normalizedApplicationId = selectedApplicationId.value.trim();
    const normalizedNickname = nickname.value.trim();
    const normalizedPassword = password.value.trim();
    const normalizedEmail = email.value.trim();
    const normalizedPhone = phone.value.trim();

    if (!normalizedApplicationId) {
        localValidationError.value = "请选择一个 application。";
        return;
    }

    if (!normalizedPassword) {
        localValidationError.value = "请输入密码。";
        return;
    }

    if (!normalizedEmail && !normalizedPhone) {
        localValidationError.value = "邮箱和手机号至少填写一个。";
        return;
    }

    localValidationError.value = null;

    emit("submit", {
        applicationId: normalizedApplicationId,
        nickname: normalizedNickname,
        password: normalizedPassword,
        email: normalizedEmail || null,
        phone: normalizedPhone || null,
    });
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

            <h3 class="text-lg font-bold">新增用户</h3>
            <p class="mt-1 text-sm text-base-content/70">
                在当前 tenant
                <span class="font-medium text-base-content">
                    {{ tenantId }}
                </span>
                下选择一个 application 创建用户。
            </p>

            <form class="mt-4 space-y-4" @submit.prevent="handleSubmit">
                <label class="form-control w-full">
                    <div class="label">
                        <span class="label-text">Application</span>
                    </div>
                    <select
                        v-model="selectedApplicationId"
                        class="select select-bordered w-full"
                        :disabled="loading || !hasApplicationOptions"
                    >
                        <option value="" disabled>
                            {{
                                hasApplicationOptions
                                    ? "请选择 application"
                                    : "当前 tenant 下暂无 application"
                            }}
                        </option>
                        <option
                            v-for="application in applications"
                            :key="application.id"
                            :value="application.id"
                        >
                            {{ application.label }}
                        </option>
                    </select>
                </label>

                <label class="form-control w-full">
                    <div class="label">
                        <span class="label-text">Nickname</span>
                    </div>
                    <div class="join w-full">
                        <input
                            v-model="nickname"
                            type="text"
                            class="input input-bordered join-item w-full"
                            placeholder="可选，不填写则由后端自动生成"
                            :disabled="loading"
                        />
                        <button
                            type="button"
                            class="btn join-item"
                            :disabled="loading"
                            aria-label="重新随机生成昵称"
                            title="重新随机"
                            @click="nickname = generateNickname()"
                        >
                            <CasinoIcon class="h-5 w-5" />
                        </button>
                    </div>
                </label>

                <label class="form-control w-full">
                    <div class="label">
                        <span class="label-text">Password</span>
                    </div>
                    <input
                        v-model="password"
                        type="password"
                        class="input input-bordered w-full"
                        placeholder="请输入用户密码"
                        :disabled="loading"
                    />
                </label>

                <label class="form-control w-full">
                    <div class="label">
                        <span class="label-text">Email</span>
                    </div>
                    <input
                        v-model="email"
                        type="email"
                        class="input input-bordered w-full"
                        placeholder="可选，邮箱和手机号至少填写一个"
                        :disabled="loading"
                    />
                </label>

                <label class="form-control w-full">
                    <div class="label">
                        <span class="label-text">Phone</span>
                    </div>
                    <input
                        v-model="phone"
                        type="text"
                        class="input input-bordered w-full"
                        placeholder="可选，邮箱和手机号至少填写一个"
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
                    :disabled="loading || !hasApplicationOptions"
                >
                    创建用户
                </button>
            </form>
        </div>

        <form method="dialog" class="modal-backdrop">
            <button aria-label="Close"></button>
        </form>
    </dialog>
</template>
