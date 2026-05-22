<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import CreateTenantModal from "./modals/CreateTenantModal.vue";
import { useAuthStore } from "../stores/auth";
import { useTenantStore } from "../stores/tenant";

const authStore = useAuthStore();
const tenantStore = useTenantStore();
const route = useRoute();
const router = useRouter();
const {
    currentTenant,
    currentTenantId,
    hasTenants,
    tenants,
    tenantsError,
    tenantsLoading,
} = storeToRefs(tenantStore);
const search = ref("");
const createTenantModalOpen = ref(false);

const isLoggedIn = computed(() => authStore.isLoggedIn);
const routeTenantId = computed(() => {
    const raw = route.params.tenantId;
    return typeof raw === "string" ? raw : "";
});
const filteredTenants = computed(() => {
    const keyword = search.value.trim().toLowerCase();
    if (!keyword) {
        return tenants.value;
    }

    return tenants.value.filter((tenant) => {
        return (
            tenant.id.toLowerCase().includes(keyword) ||
            tenant.comment?.toLowerCase().includes(keyword)
        );
    });
});
const buttonLabel = computed(() => currentTenant.value?.id || "选择 tenant");
const buttonSubtitle = computed(
    () => currentTenant.value?.comment?.trim() || "切换当前的 Tenant",
);

async function selectTenant(nextTenantId: string) {
    if (!nextTenantId || nextTenantId === currentTenantId.value) {
        return;
    }

    tenantStore.syncCurrentTenant(nextTenantId);
    search.value = "";

    await router.push({
        name: "applications",
        params: { tenantId: nextTenantId },
    });
}

function openCreateTenantModal(): void {
    createTenantModalOpen.value = true;
}

function closeCreateTenantModal(): void {
    createTenantModalOpen.value = false;
}

async function handleTenantCreated(tenantId: string): Promise<void> {
    createTenantModalOpen.value = false;
    search.value = "";

    await router.push({
        name: "applications",
        params: { tenantId },
    });
}

watch(
    () => authStore.isLoggedIn,
    (loggedIn) => {
        if (!loggedIn) {
            tenantStore.clearTenantState();
            return;
        }

        void tenantStore.loadTenants();
    },
);

watch(
    () => routeTenantId.value,
    (tenantId) => {
        if (tenantId) {
            tenantStore.syncCurrentTenant(tenantId);
            return;
        }

        tenantStore.ensureCurrentTenant();
    },
);

onMounted(() => {
    if (routeTenantId.value) {
        tenantStore.syncCurrentTenant(routeTenantId.value);
    } else {
        tenantStore.ensureCurrentTenant();
    }
    void tenantStore.loadTenants();
});
</script>

<template>
    <div v-if="isLoggedIn" class="hidden items-center gap-2 md:flex">
        <div class="dropdown dropdown-end">
            <div
                tabindex="0"
                role="button"
                class="btn btn-ghost h-auto min-h-0 gap-3 rounded-box border border-base-200 bg-base-100/90 px-3 py-2 text-left"
            >
                <div class="flex flex-col items-start leading-tight">
                    <span
                        class="max-w-40 truncate text-sm font-medium text-base-content"
                    >
                        {{ buttonLabel }}
                    </span>
                    <span
                        class="max-w-40 truncate text-xs text-base-content/55"
                    >
                        {{ tenantsLoading ? "加载中..." : buttonSubtitle }}
                    </span>
                </div>
            </div>

            <div
                tabindex="0"
                class="dropdown-content mt-2 w-88 rounded-box border border-base-200 bg-base-100 p-3 shadow-xl"
            >
                <label class="input input-sm w-full">
                    <input
                        v-model="search"
                        type="text"
                        placeholder="搜索 tenant id / comment"
                    />
                </label>

                <div
                    v-if="tenantsError"
                    class="alert alert-error alert-soft mt-3"
                >
                    <span>{{ tenantsError }}</span>
                </div>

                <div v-else-if="tenantsLoading" class="mt-3 space-y-2">
                    <div class="skeleton h-12 w-full"></div>
                    <div class="skeleton h-12 w-full"></div>
                </div>

                <div
                    v-else-if="filteredTenants.length === 0"
                    class="mt-3 rounded-box border border-dashed border-base-300 px-3 py-4 text-sm text-base-content/60"
                >
                    {{
                        hasTenants
                            ? "没有匹配的 tenant。"
                            : "当前还没有可用 tenant。"
                    }}
                </div>

                <div v-else class="mt-3 max-h-80 overflow-y-auto">
                    <button
                        v-for="tenant in filteredTenants"
                        :key="tenant.id"
                        type="button"
                        class="flex w-full items-start justify-between rounded-box px-3 py-2 text-left transition hover:bg-base-200"
                        :class="
                            tenant.id === currentTenantId ? 'bg-base-200' : ''
                        "
                        @click="selectTenant(tenant.id)"
                    >
                        <div class="min-w-0">
                            <div
                                class="truncate text-sm font-medium text-base-content"
                            >
                                {{ tenant.id }}
                            </div>
                            <div class="truncate text-xs text-base-content/60">
                                {{ tenant.comment || "暂无备注" }}
                            </div>
                        </div>
                        <span
                            v-if="tenant.id === currentTenantId"
                            class="badge badge-neutral badge-sm"
                        >
                            当前
                        </span>
                    </button>
                </div>

                <div class="mt-3 border-t border-base-200 pt-3">
                    <button
                        type="button"
                        class="btn btn-primary btn-sm w-full"
                        @click="openCreateTenantModal"
                    >
                        创建租户
                    </button>
                </div>
            </div>
        </div>

        <span
            v-if="tenantsError"
            class="max-w-40 truncate text-xs text-error"
            :title="tenantsError"
        >
            tenant 加载失败
        </span>

        <CreateTenantModal
            :open="createTenantModalOpen"
            @close="closeCreateTenantModal"
            @created="handleTenantCreated"
        />
    </div>
</template>
