<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, ref, watch } from "vue";
import { useRoute } from "vue-router";
import type { SecretVO } from "@oceaniam/sdk";
import { useToast } from "vue-toastification";
import CreateSecretModal from "../components/CreateSecretModal.vue";
import SecretTable from "../components/SecretTable.vue";
import EntityListPage from "../components/EntityListPage.vue";
import { useTenantStore } from "../stores/tenant";
import { getClient } from "../utils/api-client";

const route = useRoute();
const tenantStore = useTenantStore();
const toast = useToast();
const {
    applications,
    applicationsLoading,
    currentTenantId,
    hasTenants,
    tenantsLoading,
} = storeToRefs(tenantStore);
const secrets = ref<SecretVO[]>([]);
const secretsLoading = ref(false);
const secretsError = ref<string | null>(null);
const createSecretLoading = ref(false);
const createSecretError = ref<string | null>(null);
const isCreateDialogOpen = ref(false);
const createdSecret = ref<SecretVO | null>(null);
const requestId = ref(0);
const currentPage = ref(1);
const perPage = 25;
const totalSecrets = ref(0);
const hasNextPage = ref(false);

const tenantId = computed(() => {
    const rawTenantId = route.params.tenantId;
    return typeof rawTenantId === "string" ? rawTenantId : "";
});

const applicationComments = computed(() =>
    Object.fromEntries(
        applications.value.map((application) => [
            application.id,
            application.comment?.trim() || "暂无备注",
        ]),
    ),
);

const totalPages = computed(() =>
    totalSecrets.value === 0 ? 1 : Math.ceil(totalSecrets.value / perPage),
);

const canGoToPreviousPage = computed(() => currentPage.value > 1);

const canGoToNextPage = computed(
    () => hasNextPage.value && currentPage.value < totalPages.value,
);

const paginationSummary = computed(() => {
    if (totalSecrets.value === 0 || secrets.value.length === 0) {
        return "第 0 - 0 条，共 0 条";
    }

    const start = (currentPage.value - 1) * perPage + 1;
    const end = start + secrets.value.length - 1;
    return `第 ${start} - ${end} 条，共 ${totalSecrets.value} 条`;
});

const summaryText = computed(() => {
    const activeTenantId = tenantId.value || currentTenantId.value;

    if (tenantsLoading.value) {
        return "正在恢复 tenant 上下文...";
    }

    if (!activeTenantId) {
        return hasTenants.value ? "请先选择 tenant。" : "当前没有可用 tenant。";
    }

    if (applicationsLoading.value || secretsLoading.value) {
        return `正在加载 ${activeTenantId} 下的 API Secret 列表...`;
    }

    return `当前共 ${totalSecrets.value} 个 API Secret`;
});

function resetSecretsState(): void {
    secrets.value = [];
    secretsLoading.value = false;
    secretsError.value = null;
    totalSecrets.value = 0;
    hasNextPage.value = false;
}

function openCreateDialog(): void {
    isCreateDialogOpen.value = true;
    createSecretError.value = null;
}

function closeCreateDialog(): void {
    isCreateDialogOpen.value = false;
}

async function loadApiSecrets(nextTenantId: string): Promise<void> {
    const normalizedTenantId = nextTenantId.trim();
    tenantStore.syncCurrentTenant(normalizedTenantId);
    resetSecretsState();

    if (!normalizedTenantId) {
        secretsError.value = "缺少 tenant 标识，无法加载 API Secret 列表。";
        return;
    }

    const nextRequestId = requestId.value + 1;
    requestId.value = nextRequestId;
    secretsLoading.value = true;

    try {
        await tenantStore.loadApplications(normalizedTenantId);
        const response = await getClient().getSecrets({
            page: currentPage.value,
            per_page: perPage,
        });

        if (nextRequestId !== requestId.value) {
            return;
        }

        secrets.value = response.items;
        totalSecrets.value = response.page_info.total;
        hasNextPage.value = response.page_info.has_next;
    } catch (err) {
        if (nextRequestId !== requestId.value) {
            return;
        }

        secrets.value = [];
        secretsError.value =
            err instanceof Error ? err.message : "加载 API Secret 列表失败。";
    } finally {
        if (nextRequestId === requestId.value) {
            secretsLoading.value = false;
        }
    }
}

async function handleCreateSecret(): Promise<void> {
    createSecretLoading.value = true;
    createSecretError.value = null;

    try {
        const secret = await getClient().createSecret();
        createdSecret.value = secret;
        currentPage.value = 1;
        await loadApiSecrets(tenantId.value || currentTenantId.value);
        closeCreateDialog();
        toast.success("API Secret 已创建");
    } catch (err) {
        createSecretError.value =
            err instanceof Error ? err.message : "创建 API Secret 失败。";
        toast.error(createSecretError.value);
    } finally {
        createSecretLoading.value = false;
    }
}

async function copyCreatedSecret(): Promise<void> {
    const secret = createdSecret.value?.secret;
    if (!secret) {
        return;
    }

    try {
        await navigator.clipboard.writeText(secret);
        toast.success("Secret 已复制到剪贴板");
    } catch (err) {
        const message =
            err instanceof Error ? err.message : "复制 Secret 失败。";
        toast.error(message);
    }
}

async function goToPreviousPage(): Promise<void> {
    if (!canGoToPreviousPage.value || secretsLoading.value) {
        return;
    }

    currentPage.value -= 1;
    await loadApiSecrets(tenantId.value || currentTenantId.value);
}

async function goToNextPage(): Promise<void> {
    if (!canGoToNextPage.value || secretsLoading.value) {
        return;
    }

    currentPage.value += 1;
    await loadApiSecrets(tenantId.value || currentTenantId.value);
}

watch(
    tenantId,
    (nextTenantId, previousTenantId) => {
        if (nextTenantId !== previousTenantId) {
            currentPage.value = 1;
        }

        void loadApiSecrets(nextTenantId);
    },
    { immediate: true },
);
</script>

<template>
    <EntityListPage
        page-title="API Secrets"
        page-description="查看当前租户下的 API Secret 列表，展示 Secret 标识、状态、关联应用以及创建时间。"
        card-title="Secret List"
        :summary-text="summaryText"
    >
        <template #actions>
            <button
                type="button"
                class="btn btn-primary btn-sm"
                :disabled="
                    tenantsLoading ||
                    applicationsLoading ||
                    secretsLoading ||
                    !currentTenantId ||
                    !hasTenants
                "
                @click="openCreateDialog"
            >
                新增 Secret
            </button>
        </template>

        <div
            v-if="tenantsLoading || applicationsLoading || secretsLoading"
            class="space-y-3 px-6 pb-6"
        >
            <div class="skeleton h-12 w-full"></div>
            <div class="skeleton h-12 w-full"></div>
            <div class="skeleton h-12 w-full"></div>
        </div>

        <div v-else-if="secretsError" class="px-6 pb-6">
            <div class="alert alert-error alert-soft">
                <span>{{ secretsError }}</span>
            </div>
        </div>

        <div
            v-else-if="!hasTenants"
            class="px-6 pb-6 text-sm text-base-content/60"
        >
            当前没有可用 tenant。
        </div>

        <div
            v-else-if="secrets.length === 0"
            class="px-6 pb-6 text-sm text-base-content/60"
        >
            当前没有可展示的 API Secret。
        </div>

        <div v-else class="px-6 pb-6">
            <div class="space-y-4">
                <div
                    v-if="createdSecret"
                    class="rounded-box border border-success/20 bg-success/5 p-4"
                >
                    <div class="text-sm font-medium text-base-content">
                        最新创建的 Secret
                    </div>
                    <div
                        class="mt-2 font-mono text-sm text-base-content break-all"
                    >
                        {{ createdSecret.secret }}

                        <button
                            type="button"
                            class="btn btn-outline btn-xs"
                            @click="copyCreatedSecret"
                        >
                            复制 Secret
                        </button>
                    </div>
                    <div class="mt-3 flex flex-wrap items-center gap-3">
                        <div class="text-xs text-base-content/65">
                            Secret ID: {{ createdSecret.id }}
                        </div>
                    </div>
                </div>

                <SecretTable
                    :secrets="secrets"
                    :application-comments="applicationComments"
                    :pagination-summary="paginationSummary"
                    :current-page="currentPage"
                    :total-pages="totalPages"
                    :can-go-to-previous-page="canGoToPreviousPage"
                    :can-go-to-next-page="canGoToNextPage"
                    :loading="secretsLoading"
                    @previous-page="goToPreviousPage"
                    @next-page="goToNextPage"
                />
            </div>
        </div>
    </EntityListPage>

    <CreateSecretModal
        :open="isCreateDialogOpen"
        :tenant-id="currentTenantId || tenantId"
        :loading="createSecretLoading"
        :error="createSecretError"
        @close="closeCreateDialog"
        @submit="handleCreateSecret"
    />
</template>
