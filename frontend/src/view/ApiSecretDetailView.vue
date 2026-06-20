<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useToast } from "vue-toastification";
import type { SecretVO } from "@oceaniam/sdk";
import ConfirmDeleteSecretModal from "../components/modals/ConfirmDeleteSecretModal.vue";
import EntityListPage from "../components/layout/EntityListPage.vue";
import { useTenantStore } from "../stores/tenant";
import { getClient } from "../utils/api-client";

const route = useRoute();
const router = useRouter();
const tenantStore = useTenantStore();
const toast = useToast();

const secret = ref<SecretVO | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);
const requestId = ref(0);

const isDeleteDialogOpen = ref(false);
const deleteSecretError = ref<string | null>(null);
const deleteSecretLoading = ref(false);

const bindLoading = ref<Set<string>>(new Set());
const selectedAppId = ref<string>("");

const tenantId = computed(() => {
    const raw = route.params.tenantId;
    return typeof raw === "string" ? raw : "";
});

const secretId = computed(() => {
    const raw = route.params.secretId;
    return typeof raw === "string" ? raw : "";
});

const applicationComments = computed<Record<string, string>>(() => {
    const map: Record<string, string> = {};
    for (const app of tenantStore.applications) {
        map[app.id] = app.comment ?? app.id;
    }
    return map;
});

const boundAppIds = computed(() => secret.value?.application_ids ?? []);

const unboundApplications = computed(() => {
    const bound = new Set(boundAppIds.value);
    return tenantStore.applications.filter((app) => !bound.has(app.id));
});

const summaryText = computed(() => {
    if (loading.value) {
        return "正在加载 API Secret 详情...";
    }

    if (error.value) {
        return error.value;
    }

    if (!secret.value) {
        return "未加载到 API Secret 详情。";
    }

    return `API Secret ${secret.value.id}`;
});

function formatDateTime(value: string): string {
    const date = new Date(value);

    if (Number.isNaN(date.getTime())) {
        return value;
    }

    return date.toLocaleString("zh-CN", {
        hour12: false,
    });
}

function formatSecretMasked(secretVal: string): string {
    if (secretVal.length <= 12) {
        return secretVal;
    }

    return `${secretVal.slice(0, 6)}...${secretVal.slice(-4)}`;
}

async function loadSecretDetail(): Promise<void> {
    const normalizedTenantId = tenantId.value.trim();
    const normalizedSecretId = secretId.value.trim();

    secret.value = null;
    error.value = null;

    if (!normalizedTenantId || !normalizedSecretId) {
        error.value = "缺少 tenant 或 Secret 标识。";
        return;
    }

    tenantStore.syncCurrentTenant(normalizedTenantId);

    const currentRequestId = requestId.value + 1;
    requestId.value = currentRequestId;
    loading.value = true;

    try {
        const client = getClient();
        const [secretResponse, _tenantApplications] = await Promise.all([
            client.getSecret(normalizedSecretId),
            tenantStore.loadApplications(normalizedTenantId),
        ]);

        if (currentRequestId !== requestId.value) {
            return;
        }

        secret.value = secretResponse;
        const firstUnbound = unboundApplications.value[0];
        selectedAppId.value = firstUnbound?.id ?? "";
    } catch (err) {
        if (currentRequestId !== requestId.value) {
            return;
        }

        error.value =
            err instanceof Error ? err.message : "加载 API Secret 详情失败。";
    } finally {
        if (currentRequestId === requestId.value) {
            loading.value = false;
        }
    }
}

async function handleBindSecret(): Promise<void> {
    const appId = selectedAppId.value;
    if (!appId) return;

    bindLoading.value = new Set([...bindLoading.value, appId]);

    try {
        await getClient().bindSecretToApplication(secretId.value, appId);
        toast.success("Secret 已绑定到应用。");
        selectedAppId.value = "";
        secret.value = await getClient().getSecret(secretId.value);
        const firstUnbound = unboundApplications.value[0];
        selectedAppId.value = firstUnbound?.id ?? "";
    } catch (err) {
        const message =
            err instanceof Error ? err.message : "绑定 Secret 失败。";
        toast.error(message);
    } finally {
        const next = new Set(bindLoading.value);
        next.delete(appId);
        bindLoading.value = next;
    }
}

async function handleUnbindSecret(appId: string): Promise<void> {
    bindLoading.value = new Set([...bindLoading.value, appId]);

    try {
        await getClient().unbindSecretFromApplication(secretId.value, appId);
        toast.success("Secret 已解除绑定。");
        secret.value = await getClient().getSecret(secretId.value);
        const firstUnbound = unboundApplications.value[0];
        if (!selectedAppId.value && firstUnbound) {
            selectedAppId.value = firstUnbound.id;
        }
    } catch (err) {
        const message =
            err instanceof Error ? err.message : "解除绑定失败。";
        toast.error(message);
    } finally {
        const next = new Set(bindLoading.value);
        next.delete(appId);
        bindLoading.value = next;
    }
}

function handleBack(): void {
    void router.push({
        name: "api-secrets",
        params: { tenantId: tenantId.value },
    });
}

function openDeleteDialog(): void {
    deleteSecretError.value = null;
    isDeleteDialogOpen.value = true;
}

function closeDeleteDialog(): void {
    isDeleteDialogOpen.value = false;
}

async function handleDeleteSecret(): Promise<void> {
    deleteSecretLoading.value = true;
    deleteSecretError.value = null;

    try {
        await getClient().deleteSecret(secretId.value);
        toast.success("API Secret 已删除");
        isDeleteDialogOpen.value = false;
        void router.replace({
            name: "api-secrets",
            params: { tenantId: tenantId.value },
        });
    } catch (err) {
        deleteSecretError.value =
            err instanceof Error ? err.message : "删除 API Secret 失败。";
        toast.error(deleteSecretError.value);
    } finally {
        deleteSecretLoading.value = false;
    }
}

watch(
    [tenantId, secretId],
    () => {
        void loadSecretDetail();
    },
    { immediate: true },
);
</script>

<template>
    <EntityListPage
        page-title="API Secret 详情"
        page-description="查看 API Secret 的详细信息、状态及关联应用。"
        card-title="Secret Info"
        :summary-text="summaryText"
    >
        <template #actions>
            <button
                type="button"
                class="btn btn-outline btn-sm"
                @click="handleBack"
            >
                返回列表
            </button>
        </template>

        <div v-if="loading" class="space-y-3 px-6 pb-6">
            <div class="skeleton h-12 w-full"></div>
            <div class="skeleton h-12 w-full"></div>
            <div class="skeleton h-12 w-full"></div>
        </div>

        <div v-else-if="error" class="px-6 pb-6">
            <div class="alert alert-error alert-soft">
                <span>{{ error }}</span>
            </div>
        </div>

        <div v-else-if="secret" class="px-6 pb-6">
            <div class="space-y-6">
                <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                    <div>
                        <div class="text-sm text-base-content/60">
                            Secret ID
                        </div>
                        <div class="mt-1 font-mono text-sm text-base-content">
                            {{ secret.id }}
                        </div>
                    </div>

                    <div>
                        <div class="text-sm text-base-content/60">Secret</div>
                        <div class="mt-1 font-mono text-sm text-base-content">
                            {{ formatSecretMasked(secret.secret) }}
                        </div>
                    </div>

                    <div>
                        <div class="text-sm text-base-content/60">
                            Created At
                        </div>
                        <div class="mt-1 text-sm text-base-content">
                            {{ formatDateTime(secret.created_at) }}
                        </div>
                    </div>

                    <div>
                        <div class="text-sm text-base-content/60">Status</div>
                        <div class="mt-1">
                            <span
                                class="badge badge-sm"
                                :class="
                                    secret.revoked_at
                                        ? 'badge-error badge-soft'
                                        : 'badge-success badge-soft'
                                "
                            >
                                {{ secret.revoked_at ? "已撤销" : "有效" }}
                            </span>
                        </div>
                    </div>

                    <div v-if="secret.revoked_at">
                        <div class="text-sm text-base-content/60">
                            Revoked At
                        </div>
                        <div class="mt-1 text-sm text-base-content">
                            {{ formatDateTime(secret.revoked_at) }}
                        </div>
                    </div>
                </div>

                <div>
                    <div class="text-sm text-base-content/60 font-medium">
                        关联应用
                    </div>

                    <div
                        v-if="boundAppIds.length === 0"
                        class="mt-2 text-sm text-base-content/60"
                    >
                        未关联应用
                    </div>

                    <div v-else class="mt-2 space-y-2">
                        <div
                            v-for="appId in boundAppIds"
                            :key="appId"
                            class="flex items-center gap-2 rounded-lg border border-base-200 px-3 py-2"
                        >
                            <div class="flex-1 text-sm">
                                <span class="font-mono text-base-content">
                                    {{ appId }}
                                </span>
                                <span
                                    v-if="applicationComments[appId]"
                                    class="ml-1 text-base-content/60"
                                >
                                    ({{ applicationComments[appId] }})
                                </span>
                            </div>
                            <button
                                type="button"
                                class="btn btn-error btn-outline btn-xs"
                                :class="{
                                    loading: bindLoading.has(appId),
                                }"
                                :disabled="bindLoading.has(appId)"
                                @click="handleUnbindSecret(appId)"
                            >
                                解绑
                            </button>
                        </div>
                    </div>

                    <div
                        v-if="unboundApplications.length > 0"
                        class="mt-3 flex items-center gap-2"
                    >
                        <select
                            v-model="selectedAppId"
                            class="select select-bordered select-sm flex-1"
                        >
                            <option value="" disabled>
                                选择应用...
                            </option>
                            <option
                                v-for="app in unboundApplications"
                                :key="app.id"
                                :value="app.id"
                            >
                                {{ app.id }}
                                <template v-if="app.comment">
                                    ({{ app.comment }})
                                </template>
                            </option>
                        </select>
                        <button
                            type="button"
                            class="btn btn-primary btn-sm"
                            :disabled="!selectedAppId"
                            @click="handleBindSecret"
                        >
                            绑定
                        </button>
                    </div>
                </div>

                <div class="border-t border-base-200 pt-4">
                    <button
                        type="button"
                        class="btn btn-error"
                        @click="openDeleteDialog"
                    >
                        删除 Secret
                    </button>
                </div>
            </div>
        </div>
    </EntityListPage>

    <ConfirmDeleteSecretModal
        :open="isDeleteDialogOpen"
        :secret-id="secretId"
        :loading="deleteSecretLoading"
        :error="deleteSecretError"
        @close="closeDeleteDialog"
        @confirm="handleDeleteSecret"
    />
</template>
