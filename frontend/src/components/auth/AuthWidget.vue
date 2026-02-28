<script setup lang="ts">
import { computed } from "vue";
import { authState, logout } from "../../auth";

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

const avatarText = computed(() => {
    if (!authState.user?.displayName) return "?";
    return authState.user.displayName.trim().slice(0, 1).toUpperCase();
});

function openLogin() {
    emit("open-login");
}
</script>

<template>
    <div :class="props.variant === 'sidebar' ? 'w-full' : ''">
        <template v-if="authState.user">
            <div
                v-if="props.variant === 'sidebar'"
                class="flex items-center justify-between gap-3"
            >
                <div class="flex min-w-0 items-center gap-3">
                    <div class="avatar placeholder">
                        <div class="w-10 rounded-full bg-neutral text-neutral-content">
                            <span class="text-sm">{{ avatarText }}</span>
                        </div>
                    </div>

                    <div class="min-w-0">
                        <div class="truncate text-sm font-medium">
                            {{ authState.user.displayName }}
                        </div>
                        <div class="truncate text-xs opacity-70">
                            {{ authState.user.username }}
                        </div>
                    </div>
                </div>

                <button type="button" class="btn btn-ghost btn-sm" @click="logout">
                    退出
                </button>
            </div>

            <div v-else class="dropdown dropdown-end">
                <button type="button" class="btn btn-ghost btn-sm gap-2">
                    <div class="avatar placeholder">
                        <div class="w-8 rounded-full bg-neutral text-neutral-content">
                            <span class="text-xs">{{ avatarText }}</span>
                        </div>
                    </div>
                    <span class="hidden max-w-40 truncate sm:inline">
                        {{ authState.user.displayName }}
                    </span>
                </button>

                <ul
                    class="dropdown-content menu z-[1] w-56 rounded-box bg-base-100 p-2 shadow"
                >
                    <li class="menu-title">
                        <span class="truncate">{{ authState.user.username }}</span>
                    </li>
                    <li>
                        <button type="button" @click="logout">退出</button>
                    </li>
                </ul>
            </div>
        </template>

        <button
            v-else
            type="button"
            class="btn btn-primary btn-sm"
            :class="props.variant === 'sidebar' ? 'w-full' : ''"
            @click="openLogin"
        >
            登录
        </button>
    </div>
</template>
