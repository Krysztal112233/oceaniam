<script setup lang="ts">
import { computed, ref } from "vue";
import { useToast } from "vue-toastification";
import { logout } from "../../utils/auth.ts";
import { useAuthStore } from "../../stores/auth";

const props = withDefaults(
    defineProps<{
        variant?: "sidebar" | "navbar";
    }>(),
    {
        variant: "sidebar",
    },
);

const emit = defineEmits<{
    (event: "open-login"): void;
}>();

const authStore = useAuthStore();
const toast = useToast();
const isBusy = ref(false);

const username = computed(() => authStore.username?.trim() || "");
const displayName = computed(() => username.value || "已登录");
const isLoggedIn = computed(() => authStore.isLoggedIn);

const avatarText = computed(() => {
    if (!displayName.value) return "?";
    return displayName.value.trim().slice(0, 1).toUpperCase();
});

function openLogin() {
    emit("open-login");
}

async function handleLogout() {
    isBusy.value = true;
    try {
        await logout();
        toast.success("已退出登录。");
    } catch {
        // errors are swallowed inside logout; this is defensive
    } finally {
        isBusy.value = false;
    }
}
</script>

<template>
    <div :class="props.variant === 'sidebar' ? 'w-full' : ''">
        <template v-if="isLoggedIn">
            <div
                v-if="props.variant === 'sidebar'"
                class="flex items-center justify-between gap-3"
            >
                <div class="flex min-w-0 items-center gap-3">
                    <div class="avatar placeholder">
                        <div
                            class="w-10 rounded-full bg-neutral text-neutral-content"
                        >
                            <span class="text-sm">{{ avatarText }}</span>
                        </div>
                    </div>

                    <div class="min-w-0">
                        <div class="truncate text-sm font-medium">
                            {{ displayName }}
                        </div>
                        <div class="truncate text-xs opacity-70">
                            {{ username || "未设置" }}
                        </div>
                    </div>
                </div>

                <button
                    type="button"
                    class="btn btn-ghost btn-sm"
                    :class="{ loading: isBusy }"
                    :disabled="isBusy"
                    @click="handleLogout"
                >
                    退出
                </button>
            </div>

            <div v-else class="dropdown dropdown-end">
                <div
                    tabindex="0"
                    role="button"
                    class="btn btn-ghost btn-sm gap-2"
                >
                    <div class="avatar placeholder">
                        <div
                            class="w-8 rounded-full bg-neutral text-neutral-content"
                        >
                            <span class="text-xs">{{ avatarText }}</span>
                        </div>
                    </div>
                    <span class="hidden max-w-40 truncate sm:inline">
                        {{ displayName }}
                    </span>
                </div>

                <ul
                    tabindex="0"
                    class="dropdown-content menu mt-2 w-56 rounded-box bg-base-100 p-2 shadow"
                >
                    <li class="menu-title">
                        <span class="truncate">{{ username || "未设置" }}</span>
                    </li>
                    <li>
                        <button
                            type="button"
                            :class="{ loading: isBusy }"
                            :disabled="isBusy"
                            @click="handleLogout"
                        >
                            退出
                        </button>
                    </li>
                </ul>
            </div>
        </template>

        <button
            v-else
            type="button"
            class="btn btn-primary btn-sm"
            :class="props.variant === 'sidebar' ? 'w-full' : ''"
            :disabled="isBusy"
            @click="openLogin"
        >
            登录
        </button>
    </div>
</template>
