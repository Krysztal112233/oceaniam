<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, ref, watch } from "vue";
import { useRoute } from "vue-router";
import {
    type CreateApplicationUserRequest,
    type SearchApplicationUsersQuery,
} from "@oceaniam/sdk";
import { useToast } from "vue-toastification";
import CreateUserModal from "../components/CreateUserModal.vue";
import EntityListPage from "../components/EntityListPage.vue";
import { useAuthStore } from "../stores/auth";
import { useTenantStore } from "../stores/tenant";
import { getClient } from "../utils/api-client";

type UserItem = {
    id: string;
    nickname: string;
    email: string | null;
    phone: string | null;
};

const authStore = useAuthStore();
const route = useRoute();
const tenantStore = useTenantStore();
const toast = useToast();
const { currentTenantId, hasTenants, tenantsLoading } =
    storeToRefs(tenantStore);

const tenantId = computed(() => {
    const raw = route.params.tenantId;
    return typeof raw === "string" ? raw : "";
});

const applicationId = computed(() => {
    const raw = route.params.applicationId;
    return typeof raw === "string" ? raw : "";
});

const users = ref<UserItem[]>([]);
const usersLoading = ref(false);
const usersError = ref<string | null>(null);
const usersRequestId = ref(0);
const isCreateDialogOpen = ref(false);
const createUserLoading = ref(false);
const createUserError = ref<string | null>(null);
const searchKeyword = ref("");
const filterField = ref<"id" | "nickname" | "email" | "phone">("nickname");
const currentPage = ref(1);
const perPage = 25;
const totalUsers = ref(0);
const hasNextPage = ref(false);

const canOpenCreateDialog = computed(
    () =>
        authStore.isLoggedIn &&
        !!tenantId.value &&
        !!applicationId.value &&
        hasTenants.value &&
        !tenantsLoading.value &&
        !usersLoading.value,
);

const totalUsersCount = computed(() => totalUsers.value);
const activeUsersCount = computed(() => "--");
const hasActiveSearch = computed(() => searchKeyword.value.trim().length > 0);
const totalPages = computed(() =>
    totalUsers.value === 0 ? 1 : Math.ceil(totalUsers.value / perPage),
);
const canGoToPreviousPage = computed(() => currentPage.value > 1);
const canGoToNextPage = computed(
    () => hasNextPage.value && currentPage.value < totalPages.value,
);
const paginationSummary = computed(() => {
    if (totalUsers.value === 0 || users.value.length === 0) {
        return "第 0 - 0 条，共 0 条";
    }

    const start = (currentPage.value - 1) * perPage + 1;
    const end = start + users.value.length - 1;
    return `第 ${start} - ${end} 条，共 ${totalUsers.value} 条`;
});

function mapUsers(
    items: Array<{
        id: string;
        nickname: string;
        email: string | null;
        phone: string | null;
    }>,
): UserItem[] {
    return items.map((user) => ({
        id: user.id,
        nickname: user.nickname,
        email: user.email,
        phone: user.phone,
    }));
}

function buildUserSearchQuery(
    keyword: string,
    field: "id" | "nickname" | "email" | "phone",
): SearchApplicationUsersQuery {
    if (field === "id") {
        return { by_id: keyword };
    }

    if (field === "email") {
        return { by_email: keyword };
    }

    if (field === "phone") {
        return { by_phone: keyword };
    }

    return { by_nickname: keyword };
}

async function applySearch(): Promise<void> {
    currentPage.value = 1;
    await loadUsers(tenantId.value, applicationId.value);
}

function clearUsersState(): void {
    users.value = [];
    usersLoading.value = false;
    usersError.value = null;
    createUserError.value = null;
    createUserLoading.value = false;
    isCreateDialogOpen.value = false;
    searchKeyword.value = "";
    filterField.value = "nickname";
    currentPage.value = 1;
    totalUsers.value = 0;
    hasNextPage.value = false;
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

async function loadUsers(
    tenantId: string,
    applicationId: string,
): Promise<void> {
    const normalizedTenantId = tenantId.trim();
    const normalizedApplicationId = applicationId.trim();
    const keyword = searchKeyword.value.trim();

    if (!normalizedTenantId || !normalizedApplicationId) {
        clearUsersState();
        usersError.value = "缺少 tenant 或 application 标识。";
        return;
    }

    tenantStore.syncCurrentTenant(normalizedTenantId);

    const requestId = usersRequestId.value + 1;
    usersRequestId.value = requestId;
    usersLoading.value = true;
    usersError.value = null;

    try {
        const response = keyword
            ? await getClient().searchApplicationUsers(
                  normalizedTenantId,
                  normalizedApplicationId,
                  {
                      ...buildUserSearchQuery(keyword, filterField.value),
                      page: currentPage.value,
                      per_page: perPage,
                  },
              )
            : await getClient().getApplicationUsers(
                  normalizedTenantId,
                  normalizedApplicationId,
                  {
                      page: currentPage.value,
                      per_page: perPage,
                  },
              );

        if (requestId !== usersRequestId.value) {
            return;
        }

        users.value = mapUsers(response.items);
        totalUsers.value = response.page_info.total;
        hasNextPage.value = response.page_info.has_next;
    } catch (err) {
        if (requestId !== usersRequestId.value) {
            return;
        }

        users.value = [];
        totalUsers.value = 0;
        hasNextPage.value = false;
        usersError.value =
            err instanceof Error
                ? err.message
                : keyword
                  ? "搜索 application 用户失败。"
                  : "加载 application 用户列表失败。";
    } finally {
        if (requestId === usersRequestId.value) {
            usersLoading.value = false;
        }
    }
}

function openCreateDialog(): void {
    createUserError.value = null;
    if (!canOpenCreateDialog.value) {
        return;
    }
    isCreateDialogOpen.value = true;
}

function closeCreateDialog(): void {
    if (createUserLoading.value) {
        return;
    }

    isCreateDialogOpen.value = false;
    createUserError.value = null;
}

async function handleCreateUser(
    payload: CreateApplicationUserRequest & { applicationId: string },
): Promise<void> {
    const normalizedTenantId = tenantId.value.trim();
    const normalizedApplicationId = applicationId.value.trim();

    if (!normalizedTenantId || !normalizedApplicationId) {
        createUserError.value =
            "缺少 tenant 或 application 标识，无法创建用户。";
        return;
    }

    createUserLoading.value = true;
    createUserError.value = null;

    try {
        await getClient().createApplicationUser(
            normalizedTenantId,
            normalizedApplicationId,
            {
                nickname: payload.nickname,
                password: payload.password,
                email: payload.email,
                phone: payload.phone,
            },
        );

        isCreateDialogOpen.value = false;
        currentPage.value = 1;
        toast.success("用户已创建");
        await loadUsers(normalizedTenantId, normalizedApplicationId);
    } catch (err) {
        createUserError.value =
            err instanceof Error ? err.message : "创建用户失败。";
        toast.error(createUserError.value);
    } finally {
        createUserLoading.value = false;
    }
}

async function goToPreviousPage(): Promise<void> {
    if (!canGoToPreviousPage.value || usersLoading.value) {
        return;
    }

    currentPage.value -= 1;
    await loadUsers(tenantId.value, applicationId.value);
}

async function goToNextPage(): Promise<void> {
    if (!canGoToNextPage.value || usersLoading.value) {
        return;
    }

    currentPage.value += 1;
    await loadUsers(tenantId.value, applicationId.value);
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
    (nextTenantId) => {
        if (!authStore.isLoggedIn) {
            resetPageState();
            return;
        }

        if (tenantId.value && nextTenantId !== tenantId.value) {
            tenantStore.syncCurrentTenant(tenantId.value);
            return;
        }

        void loadUsers(tenantId.value, applicationId.value);
    },
    { immediate: true },
);

watch(
    () => [tenantId.value, applicationId.value],
    (nextParams, previousParams) => {
        if (!authStore.isLoggedIn) {
            resetPageState();
            return;
        }

        const nextTenantId = nextParams[0] ?? "";
        const nextApplicationId = nextParams[1] ?? "";
        const previousTenantId = previousParams?.[0] ?? "";
        const previousApplicationId = previousParams?.[1] ?? "";

        if (
            nextTenantId !== previousTenantId ||
            nextApplicationId !== previousApplicationId
        ) {
            currentPage.value = 1;
        }

        void loadUsers(nextTenantId, nextApplicationId);
    },
    { immediate: true },
);
</script>

<template>
    <EntityListPage
        page-title="User Management"
        page-description="管理当前 application 下的用户列表，并在相同 application 上下文中创建新用户。"
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
                    <div class="join">
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
                            <option value="id">User ID</option>
                            <option value="nickname">Nickname</option>
                            <option value="email">Email</option>
                            <option value="phone">Phone</option>
                        </select>
                    </div>
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
                        :disabled="!canOpenCreateDialog"
                        @click="openCreateDialog"
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
                        <span>请先登录后再查看 application 用户列表。</span>
                    </div>

                    <div
                        v-else-if="tenantsLoading && !tenantId"
                        class="space-y-3"
                    >
                        <div class="skeleton h-12 w-full"></div>
                        <div class="skeleton h-12 w-full"></div>
                        <div class="skeleton h-12 w-full"></div>
                    </div>

                    <div
                        v-else-if="!tenantId || !applicationId"
                        class="alert alert-info alert-soft"
                    >
                        <span>缺少 tenant 或 application 上下文。</span>
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
                        v-else-if="users.length === 0 && hasActiveSearch"
                        class="alert alert-info alert-soft"
                    >
                        <span>没有匹配当前搜索条件的用户。</span>
                    </div>

                    <div
                        v-else-if="users.length === 0"
                        class="alert alert-info alert-soft"
                    >
                        <span>当前 application 下暂无用户。</span>
                    </div>

                    <div v-else class="rounded-box border border-base-200">
                        <div class="overflow-x-auto">
                            <table class="table min-w-180">
                                <thead>
                                    <tr class="text-sm text-base-content/70">
                                        <th class="whitespace-nowrap">
                                            User ID
                                        </th>
                                        <th class="whitespace-nowrap">
                                            Nickname
                                        </th>
                                        <th class="whitespace-nowrap">Email</th>
                                        <th class="whitespace-nowrap">Phone</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <tr v-for="user in users" :key="user.id">
                                        <td
                                            class="whitespace-nowrap font-medium"
                                        >
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

                        <div
                            class="flex flex-col gap-3 border-t border-base-200 px-4 py-4 text-sm text-base-content/70 md:flex-row md:items-center md:justify-between"
                        >
                            <div>{{ paginationSummary }}</div>

                            <div class="join">
                                <button
                                    type="button"
                                    class="btn btn-sm join-item"
                                    :disabled="
                                        !canGoToPreviousPage || usersLoading
                                    "
                                    @click="goToPreviousPage"
                                >
                                    上一页
                                </button>
                                <button
                                    type="button"
                                    class="btn btn-sm join-item"
                                    disabled
                                >
                                    第 {{ currentPage }} / {{ totalPages }} 页
                                </button>
                                <button
                                    type="button"
                                    class="btn btn-sm join-item"
                                    :disabled="!canGoToNextPage || usersLoading"
                                    @click="goToNextPage"
                                >
                                    下一页
                                </button>
                            </div>
                        </div>
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

    <CreateUserModal
        :open="isCreateDialogOpen"
        :tenant-id="tenantId"
        :loading="createUserLoading"
        :error="createUserError"
        :applications="[{ id: applicationId, label: applicationId }]"
        @close="closeCreateDialog"
        @submit="handleCreateUser"
    />
</template>
