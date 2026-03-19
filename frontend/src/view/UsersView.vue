<script setup lang="ts">
import { storeToRefs } from "pinia";
import GroupIcon from "@iconify-vue/material-symbols/group-outline-rounded";
import PersonOffIcon from "@iconify-vue/material-symbols/person-off-outline-rounded";
import ScheduleIcon from "@iconify-vue/material-symbols/schedule-outline-rounded";
import VerifiedUserIcon from "@iconify-vue/material-symbols/verified-user-outline-rounded";
import { computed, ref, watch } from "vue";
import { OceanIamClient } from "@oceaniam/sdk";
import EntityListPage from "../components/EntityListPage.vue";
import MetricsStats from "../components/MetricsStats.vue";
import MetricsStatsItem from "../components/MetricsStatsItem.vue";
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

const totalUsersValue = computed(() => {
    if (usersLoading.value) {
        return "...";
    }

    if (!authStore.isLoggedIn || !currentTenantId.value || usersError.value) {
        return "0";
    }

    return String(users.value.length);
});

const totalUsersDesc = computed(() => {
    if (usersLoading.value) {
        return "正在加载当前 tenant 下的用户";
    }

    if (!authStore.isLoggedIn) {
        return "登录后查看真实统计";
    }

    if (tenantsLoading.value && !currentTenantId.value) {
        return "正在恢复 tenant 上下文";
    }

    if (!currentTenantId.value) {
        return hasTenants.value ? "请先选择 tenant" : "当前没有可用 tenant";
    }

    if (usersError.value) {
        return "当前统计不可用";
    }

    return `${currentTenantId.value} 下的真实用户总数`;
});

const currentTenantValue = computed(() => {
    if (tenantsLoading.value && !currentTenantId.value) {
        return "恢复中";
    }

    return currentTenantId.value || "未选择";
});

const currentTenantDesc = computed(() => {
    if (tenantsLoading.value && !currentTenantId.value) {
        return "正在从持久化状态恢复 tenant 上下文";
    }

    if (!currentTenantId.value) {
        return hasTenants.value ? "请先选择 tenant" : "当前没有可用 tenant";
    }

    return `当前页面展示租户 ${currentTenantId.value} 的用户视图`;
});

const statusValue = computed(() => {
    if (usersError.value) {
        return "异常";
    }

    if (tenantsLoading.value || usersLoading.value) {
        return "加载中";
    }

    if (!authStore.isLoggedIn) {
        return "未登录";
    }

    if (!currentTenantId.value) {
        return hasTenants.value ? "待选择 tenant" : "无 tenant";
    }

    if (users.value.length === 0) {
        return "空列表";
    }

    return "已就绪";
});

const statusDesc = computed(() => {
    if (usersError.value) {
        return usersError.value;
    }

    if (tenantsLoading.value && !currentTenantId.value) {
        return "正在恢复 tenant 上下文";
    }

    if (usersLoading.value) {
        return "正在加载当前 tenant 下的用户";
    }

    if (!authStore.isLoggedIn) {
        return "登录后查看租户用户数据";
    }

    if (!currentTenantId.value) {
        return hasTenants.value ? "请先选择 tenant" : "当前没有可用 tenant";
    }

    if (users.value.length === 0) {
        return "当前 tenant 下暂无用户";
    }

    return "租户用户列表可查看，后续操作入口暂未开放";
});

const statusIcon = computed(() => {
    if (usersError.value) {
        return PersonOffIcon;
    }

    if (tenantsLoading.value || usersLoading.value) {
        return ScheduleIcon;
    }

    if (users.value.length === 0) {
        return PersonOffIcon;
    }

    return VerifiedUserIcon;
});

function clearUsersState(): void {
    users.value = [];
    usersLoading.value = false;
    usersError.value = null;
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
        const tenantUsers =
            await getClient().getTenantUsers(normalizedTenantId);

        if (requestId !== usersRequestId.value) {
            return;
        }

        users.value = tenantUsers.items.map((user) => ({
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
        card-title="Tenant Users"
    >
        <template #actions>
            <button type="button" class="btn btn-primary btn-sm" disabled>
                新增用户
            </button>
        </template>

        <div class="space-y-6 px-6 py-6">
            <MetricsStats class="border border-base-200 shadow-none">
                <MetricsStatsItem
                    title="总用户数"
                    :value="totalUsersValue"
                    :desc="totalUsersDesc"
                    :icon="GroupIcon"
                />
                <MetricsStatsItem
                    title="当前 Tenant"
                    :value="currentTenantValue"
                    :desc="currentTenantDesc"
                    :icon="VerifiedUserIcon"
                    figure-class="text-primary"
                />
                <MetricsStatsItem
                    title="当前状态"
                    :value="statusValue"
                    :desc="statusDesc"
                    :icon="statusIcon"
                    figure-class="text-secondary"
                />
            </MetricsStats>

            <section class="rounded-box border border-base-200 bg-base-100">
                <div
                    class="flex flex-col gap-3 border-b border-base-200 px-5 py-4 lg:flex-row lg:items-center lg:justify-between"
                >
                    <div class="space-y-1">
                        <h3 class="text-base font-medium text-base-content">
                            {{
                                currentTenantId
                                    ? `${currentTenantId} 用户列表`
                                    : "租户用户列表"
                            }}
                        </h3>
                        <p class="text-sm text-base-content/70">
                            当前工作区聚焦当前 tenant 下的全部用户。
                        </p>
                    </div>

                    <div class="badge badge-neutral badge-outline">
                        {{ totalUsersValue }} users
                    </div>
                </div>

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
                                <tr v-for="user in users" :key="user.id">
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
