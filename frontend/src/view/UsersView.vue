<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, ref, watch } from "vue";
import { OceanIamClient } from "@oceaniam/sdk";
import EntityListPage from "../components/EntityListPage.vue";
import { appConfig } from "../config";
import { useAuthStore } from "../stores/auth";
import { useTenantStore } from "../stores/tenant";

const authStore = useAuthStore();
const tenantStore = useTenantStore();
const { currentTenantId, hasTenants, tenantsLoading } =
    storeToRefs(tenantStore);

function getClient(): OceanIamClient {
    return new OceanIamClient({
        baseUrl: appConfig.systemBaseUrl,
        tokenGetter: () => authStore.jwt,
    });
}

const users = ref<
    Array<{
        id: string;
        nickname: string;
        email: string | null;
        phone: string | null;
    }>
>([]);
const usersLoading = ref(false);
const usersError = ref<string | null>(null);
const usersRequestId = ref(0);
const searchKeyword = ref("");
const appliedKeyword = ref("");
const filterField = ref<"all" | "id" | "nickname" | "email" | "phone">("all");

const totalUsersCount = computed(() => users.value.length);
const activeUsersCount = computed(() => "--");

const filteredUsers = computed(() => {
    const keyword = appliedKeyword.value.trim().toLowerCase();
    if (!keyword) {
        return users.value;
    }

    return users.value.filter((user) => {
        const fields =
            filterField.value === "all"
                ? [user.id, user.nickname, user.email || "", user.phone || ""]
                : [user[filterField.value] || ""];

        return fields.some((value) => value.toLowerCase().includes(keyword));
    });
});

function applySearch(): void {
    appliedKeyword.value = searchKeyword.value.trim();
}

function clearUsersState(): void {
    users.value = [];
    usersLoading.value = false;
    usersError.value = null;
    searchKeyword.value = "";
    appliedKeyword.value = "";
    filterField.value = "all";
}

function resetPageState(): void {
    clearUsersState();
}

async function ensureTenantContext(): Promise<void> {
    if (!authStore.isLoggedIn) {
        return;
    }

    if (!tenantStore.hasTenants && !tenantStore.tenantsLoading) {
        await tenantStore.loadTenants();
    }
}

async function prepareTenantUsersView(tenantId: string): Promise<void> {
    clearUsersState();

    const normalizedTenantId = tenantId.trim();
    if (!normalizedTenantId) {
        return;
    }

    const requestId = usersRequestId.value + 1;
    usersRequestId.value = requestId;
    usersLoading.value = true;

    try {
        const tenantUsersResponse =
            await getClient().getTenantUsers(normalizedTenantId);

        if (requestId !== usersRequestId.value) {
            return;
        }

        users.value = tenantUsersResponse.items.map((user) => ({
            id: user.id,
            nickname: user.nickname,
            email: user.email,
            phone: user.phone,
        }));
    } catch (err) {
        if (requestId !== usersRequestId.value) {
            return;
        }

        users.value = [];
        usersError.value =
            err instanceof Error ? err.message : "加载 tenant 用户列表失败。";
    } finally {
        if (requestId === usersRequestId.value) {
            usersLoading.value = false;
        }
    }
}

watch(
    () => authStore.isLoggedIn,
    (loggedIn) => {
        if (!loggedIn) {
            resetPageState();
            return;
        }

        void ensureTenantContext();
    },
    { immediate: true },
);

watch(
    () => currentTenantId.value,
    (tenantId) => {
        if (!authStore.isLoggedIn) {
            resetPageState();
            return;
        }

        void prepareTenantUsersView(tenantId);
    },
    { immediate: true },
);
</script>

<template>
    <EntityListPage
        page-title="Users"
        page-description="按 Tenant 维度展示当前租户下的用户视图。"
    >
        <template #actions>
            <div
                class="flex w-full flex-col gap-4 xl:flex-row xl:items-center xl:justify-between"
            >
                <div class="flex flex-col gap-3 sm:flex-row sm:items-center">
                    <label
                        class="rounded-box border border-base-200 bg-base-50 px-3 py-2"
                    >
                        <div class="label p-0">
                            <span
                                class="label-text text-xs text-base-content/60"
                            >
                                用户总量 {{ totalUsersCount }}
                            </span>
                        </div>
                    </label>

                    <label
                        class="rounded-box border border-base-200 bg-base-50 px-3 py-2"
                    >
                        <div class="label p-0">
                            <span
                                class="label-text text-xs text-base-content/60"
                            >
                                活跃总量 {{ activeUsersCount }}
                            </span>
                        </div>
                    </label>
                </div>

                <div class="flex flex-col gap-3 lg:flex-row lg:items-center">
                    <label class="input input-sm w-full lg:w-64">
                        <input
                            v-model="searchKeyword"
                            type="text"
                            placeholder="搜索用户"
                            @keydown.enter.prevent="applySearch"
                        />
                    </label>

                    <select
                        v-model="filterField"
                        class="select select-bordered select-sm w-full lg:w-40"
                    >
                        <option value="all">全部字段</option>
                        <option value="id">User ID</option>
                        <option value="nickname">Nickname</option>
                        <option value="email">Email</option>
                        <option value="phone">Phone</option>
                    </select>

                    <button
                        type="button"
                        class="btn btn-outline btn-sm"
                        @click="applySearch"
                    >
                        搜索
                    </button>

                    <button
                        type="button"
                        class="btn btn-primary btn-sm"
                        disabled
                    >
                        新增用户
                    </button>
                </div>
            </div>
        </template>

        <div class="space-y-6 px-6 py-6">
            <section class="rounded-box border border-base-200 bg-base-100">
                <div class="p-5">
                    <div
                        v-if="!authStore.isLoggedIn"
                        class="alert alert-info alert-soft"
                    >
                        <span>请先登录后再查看租户用户列表。</span>
                    </div>

                    <div
                        v-else-if="tenantsLoading && !currentTenantId"
                        class="space-y-3"
                    >
                        <div class="skeleton h-12 w-full"></div>
                        <div class="skeleton h-12 w-full"></div>
                        <div class="skeleton h-12 w-full"></div>
                    </div>

                    <div
                        v-else-if="!currentTenantId"
                        class="alert alert-info alert-soft"
                    >
                        <span>
                            {{
                                hasTenants
                                    ? "请先选择 tenant。"
                                    : "当前没有可用 tenant。"
                            }}
                        </span>
                    </div>

                    <div
                        v-else-if="usersError"
                        class="alert alert-error alert-soft"
                    >
                        <span>{{ usersError }}</span>
                    </div>

                    <div v-else-if="usersLoading" class="space-y-3">
                        <div class="skeleton h-12 w-full"></div>
                        <div class="skeleton h-12 w-full"></div>
                        <div class="skeleton h-12 w-full"></div>
                    </div>

                    <div
                        v-else-if="users.length === 0"
                        class="alert alert-info alert-soft"
                    >
                        <span>当前 tenant 下暂无用户。</span>
                    </div>

                    <div
                        v-else-if="filteredUsers.length === 0"
                        class="alert alert-info alert-soft"
                    >
                        <span>没有匹配当前搜索条件的用户。</span>
                    </div>

                    <div
                        v-else
                        class="overflow-x-auto rounded-box border border-base-200"
                    >
                        <table class="table min-w-180">
                            <thead>
                                <tr class="text-sm text-base-content/70">
                                    <th class="whitespace-nowrap">User ID</th>
                                    <th class="whitespace-nowrap">Nickname</th>
                                    <th class="whitespace-nowrap">Email</th>
                                    <th class="whitespace-nowrap">Phone</th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr
                                    v-for="user in filteredUsers"
                                    :key="user.id"
                                >
                                    <td class="whitespace-nowrap font-medium">
                                        {{ user.id }}
                                    </td>
                                    <td class="whitespace-nowrap">
                                        {{ user.nickname }}
                                    </td>
                                    <td class="whitespace-nowrap">
                                        {{ user.email || "-" }}
                                    </td>
                                    <td class="whitespace-nowrap">
                                        {{ user.phone || "-" }}
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                    </div>

                    <div class="mt-4 alert alert-warning alert-soft">
                        <span>
                            当前不提供详情、编辑或删除入口，避免暴露尚未实现的无效操作。
                        </span>
                    </div>
                </div>
            </section>
        </div>
    </EntityListPage>
</template>
