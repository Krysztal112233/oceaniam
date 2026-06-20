<script setup lang="ts">
import { computed, defineAsyncComponent, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useToast } from "vue-toastification";
import type {
    ApplicationConfigurationVO,
    ApplicationKeyVO,
    ApplicationVO,
    SecretVO,
} from "@oceaniam/sdk";
import EntityListPage from "../components/layout/EntityListPage.vue";
import { useTenantStore } from "../stores/tenant";
import { getClient } from "../utils/api-client";
import { appConfig } from "../config";
import ApplicationKeyTable from "../components/tables/ApplicationKeyTable.vue";

const ConfigEditor = defineAsyncComponent(
    () => import("../components/ConfigEditor.vue"),
);

const route = useRoute();
const router = useRouter();
const tenantStore = useTenantStore();
const toast = useToast();

const application = ref<ApplicationVO | null>(null);
const configuration = ref<ApplicationConfigurationVO | null>(null);
const commentDraft = ref("");
const commentSaving = ref(false);
const loading = ref(false);
const error = ref<string | null>(null);
const requestId = ref(0);
const keys = ref<ApplicationKeyVO[]>([]);
const loadingKeys = ref(false);
const creatingKey = ref(false);
const revokingKeyId = ref<string | null>(null);
const newlyCreatedKey = ref<ApplicationKeyVO | null>(null);
const secrets = ref<SecretVO[]>([]);
const loadingSecrets = ref(false);
const activeTab = ref<"config" | "keys" | "secrets">("config");

const tenantId = computed(() => {
    const raw = route.params.tenantId;
    return typeof raw === "string" ? raw : "";
});

const applicationId = computed(() => {
    const raw = route.params.applicationId;
    return typeof raw === "string" ? raw : "";
});

const summaryText = computed(() => {
    if (loading.value) {
        return "正在加载 application 详情...";
    }

    if (error.value) {
        return error.value;
    }

    if (!application.value) {
        return "未加载到 application 详情。";
    }

    return `Application ${application.value.id} 隶属于 tenant ${application.value.tenant_id}`;
});

const jwksUrl = computed(() => {
    const tid = tenantId.value;
    if (!tid) return null;
    const base = appConfig.systemBaseUrl.replace(/\/+$/, "");
    return `${base}/tenants/${tid}/.well-known/jwks.json`;
});

const commentSaveEnabled = computed(() => {
    const currentComment = application.value?.comment ?? "";
    return !commentSaving.value && commentDraft.value !== currentComment;
});

async function loadApplicationDetail(): Promise<void> {
    const normalizedTenantId = tenantId.value.trim();
    const normalizedApplicationId = applicationId.value.trim();

    application.value = null;
    configuration.value = null;
    keys.value = [];
    secrets.value = [];
    error.value = null;

    if (!normalizedTenantId || !normalizedApplicationId) {
        error.value = "缺少 tenant 或 application 标识。";
        return;
    }

    tenantStore.syncCurrentTenant(normalizedTenantId);

    const currentRequestId = requestId.value + 1;
    requestId.value = currentRequestId;
    loading.value = true;
    loadingKeys.value = true;
    loadingSecrets.value = true;

    try {
        const client = getClient();
        const [applicationResponse, configurationResponse] = await Promise.all([
            client.getApplication(normalizedTenantId, normalizedApplicationId),
            client.getApplicationConfiguration(
                normalizedTenantId,
                normalizedApplicationId,
            ),
        ]);

        if (currentRequestId !== requestId.value) {
            return;
        }

        application.value = applicationResponse;
        configuration.value = configurationResponse.configuration;
    } catch (err) {
        if (currentRequestId !== requestId.value) {
            return;
        }

        error.value =
            err instanceof Error ? err.message : "加载 application 详情失败。";
    } finally {
        if (currentRequestId === requestId.value) {
            loading.value = false;
        }
    }

    try {
        const client = getClient();
        const response = await client.getTenantKeys(normalizedTenantId);

        if (currentRequestId !== requestId.value) {
            return;
        }

        keys.value = response.items;
    } catch (err) {
        if (currentRequestId !== requestId.value) {
            return;
        }

        toast.error("加载密钥列表失败。");
        keys.value = [];
    } finally {
        if (currentRequestId === requestId.value) {
            loadingKeys.value = false;
        }
    }

    try {
        const client = getClient();
        const response = await client.getApplicationSecrets(
            normalizedTenantId,
            normalizedApplicationId,
        );

        if (currentRequestId !== requestId.value) {
            return;
        }

        secrets.value = response.items;
    } catch (err) {
        if (currentRequestId !== requestId.value) {
            return;
        }

        console.error("加载 API 密钥列表失败:", err);
        secrets.value = [];
    } finally {
        if (currentRequestId === requestId.value) {
            loadingSecrets.value = false;
        }
    }
}

async function goBack(): Promise<void> {
    await router.push({
        name: "applications",
        params: { tenantId: tenantId.value },
    });
}

async function handleConfigurationSubmit(
    nextConfiguration: ApplicationConfigurationVO,
): Promise<void> {
    const normalizedTenantId = tenantId.value.trim();
    const normalizedApplicationId = applicationId.value.trim();

    if (!normalizedTenantId || !normalizedApplicationId) {
        const message = "缺少 tenant 或 application 标识。";
        toast.error(message);
        throw new Error(message);
    }

    try {
        const client = getClient();
        await client.patchApplicationConfiguration(
            normalizedTenantId,
            normalizedApplicationId,
            {
                auth: {
                    token: {
                        issuer: nextConfiguration.auth.token.issuer,
                        audience: nextConfiguration.auth.token.audience,
                    },
                },
                registration: {
                    enabled: nextConfiguration.registration.enabled,
                },
            },
        );

        configuration.value = nextConfiguration;
        toast.success("Application configuration 已更新。");
    } catch (err) {
        const message =
            err instanceof Error ? err.message : "提交 application 配置失败。";
        toast.error(message);
        throw err;
    }
}

async function handleCommentSubmit(): Promise<void> {
    const normalizedTenantId = tenantId.value.trim();
    const normalizedApplicationId = applicationId.value.trim();

    if (!normalizedTenantId || !normalizedApplicationId) {
        toast.error("缺少 tenant 或 application 标识。");
        return;
    }

    commentSaving.value = true;

    try {
        const client = getClient();
        const nextComment = commentDraft.value.trim() || null;
        const updatedApplication = await client.patchApplication(
            normalizedTenantId,
            normalizedApplicationId,
            {
                comment: nextComment,
            },
        );

        application.value = {
            id: updatedApplication.id,
            tenant_id: updatedApplication.tenant_id,
            comment: updatedApplication.comment,
        };
        commentDraft.value = updatedApplication.comment ?? "";
        toast.success("Application comment 已更新。");
    } catch (err) {
        const message =
            err instanceof Error ? err.message : "更新 comment 失败。";
        toast.error(message);
    } finally {
        commentSaving.value = false;
    }
}

async function handleRotateKey(): Promise<void> {
    const normalizedTenantId = tenantId.value.trim();

    if (!normalizedTenantId) {
        toast.error("缺少 tenant 标识。");
        return;
    }

    creatingKey.value = true;

    try {
        const client = getClient();
        await client.rotateTenantKey(normalizedTenantId);

        const keysResponse = await client.getTenantKeys(normalizedTenantId);
        keys.value = keysResponse.items;

        const latestActive = keys.value
            .filter((k: { status: string }) => k.status === "Active")
            .sort(
                (a: { activated_at: string }, b: { activated_at: string }) =>
                    new Date(b.activated_at).getTime() -
                    new Date(a.activated_at).getTime(),
            )[0];
        newlyCreatedKey.value = latestActive ?? null;
        toast.success("新密钥已生成。");
    } catch (err) {
        const message = err instanceof Error ? err.message : "轮换密钥失败。";
        toast.error(message);
    } finally {
        creatingKey.value = false;
    }
}

async function handleRevokeKey(keyId: string): Promise<void> {
    const normalizedTenantId = tenantId.value.trim();

    if (!normalizedTenantId) {
        toast.error("缺少 tenant 标识。");
        return;
    }

    revokingKeyId.value = keyId;

    try {
        const client = getClient();
        await client.revokeTenantKey(normalizedTenantId, keyId);

        toast.success("密钥已吊销。");

        const keysResponse = await client.getTenantKeys(normalizedTenantId);
        keys.value = keysResponse.items;
        newlyCreatedKey.value = null;
    } catch (err) {
        const message = err instanceof Error ? err.message : "吊销密钥失败。";
        toast.error(message);
    } finally {
        revokingKeyId.value = null;
    }
}

async function copyToClipboard(text: string): Promise<void> {
    try {
        await navigator.clipboard.writeText(text);
        toast.success("已复制到剪贴板。");
    } catch {
        toast.error("复制失败。");
    }
}

watch(
    () => application.value?.comment,
    (nextComment) => {
        commentDraft.value = nextComment ?? "";
    },
    { immediate: true },
);

watch(
    () => [tenantId.value, applicationId.value],
    () => {
        void loadApplicationDetail();
    },
    { immediate: true },
);
</script>

<template>
    <EntityListPage
        page-title="Application Detail"
        page-description="查看当前 application 的基础信息，管理配置与 JWT 签名密钥。"
        card-title="Application Information"
        :summary-text="summaryText"
    >
        <template #actions>
            <div class="flex flex-wrap items-center justify-end gap-2">
                <button
                    type="button"
                    class="btn btn-outline btn-sm"
                    @click="goBack"
                >
                    返回 Applications
                </button>
            </div>
        </template>

        <div v-if="loading" class="space-y-3 px-6 py-6">
            <div class="skeleton h-24 w-full"></div>
            <div class="skeleton h-24 w-full"></div>
        </div>

        <div v-else-if="error" class="px-6 py-6">
            <div class="alert alert-error alert-soft">
                <span>{{ error }}</span>
            </div>
        </div>

        <div v-else-if="application" class="space-y-6 px-6 py-6">
            <section class="grid gap-4 lg:grid-cols-2">
                <div class="rounded-box border border-base-200 bg-base-50 p-5">
                    <div class="label px-0 pt-0">
                        <span class="label-text text-xs text-base-content/60">
                            Application ID
                        </span>
                    </div>
                    <p class="break-all font-mono text-sm text-base-content">
                        {{ application.id }}
                    </p>
                </div>

                <div class="rounded-box border border-base-200 bg-base-50 p-5">
                    <div class="label px-0 pt-0">
                        <span class="label-text text-xs text-base-content/60">
                            Tenant ID
                        </span>
                    </div>
                    <p class="break-all font-mono text-sm text-base-content">
                        {{ application.tenant_id }}
                    </p>
                </div>

                <div
                    class="rounded-box border border-base-200 bg-base-50 p-5 lg:col-span-2"
                >
                    <div class="label px-0 pt-0">
                        <span class="label-text text-xs text-base-content/60">
                            Comment
                        </span>
                    </div>
                    <div class="flex items-center gap-3">
                        <input
                            v-model="commentDraft"
                            class="input input-bordered flex-1"
                            placeholder="无"
                        />
                        <button
                            type="button"
                            class="btn btn-primary"
                            :disabled="!commentSaveEnabled"
                            :class="{ loading: commentSaving }"
                            @click="handleCommentSubmit"
                        >
                            保存
                        </button>
                    </div>
                </div>

                <div
                    class="rounded-box border border-base-200 bg-base-50 p-5 lg:col-span-2"
                >
                    <div class="label px-0 pt-0">
                        <span class="label-text text-xs text-base-content/60">
                            .well-known JWKS
                        </span>
                    </div>
                    <div class="flex items-center gap-3">
                        <code
                            class="flex-1 break-all font-mono text-sm text-base-content"
                            >{{ jwksUrl }}</code
                        >
                        <button
                            type="button"
                            class="btn btn-outline btn-sm"
                            :disabled="!jwksUrl"
                            @click="copyToClipboard(jwksUrl ?? '')"
                        >
                            复制
                        </button>
                    </div>
                </div>
            </section>

            <div role="tablist" class="tabs tabs-lift">
                <button
                    role="tab"
                    class="tab"
                    :class="{ 'tab-active': activeTab === 'config' }"
                    @click="activeTab = 'config'"
                >
                    配置
                </button>
                <button
                    role="tab"
                    class="tab"
                    :class="{ 'tab-active': activeTab === 'keys' }"
                    @click="activeTab = 'keys'"
                >
                    密钥管理
                </button>
                <button
                    role="tab"
                    class="tab"
                    :class="{ 'tab-active': activeTab === 'secrets' }"
                    @click="activeTab = 'secrets'"
                >
                    API 密钥
                </button>
            </div>

            <div v-show="activeTab === 'config'" class="pt-4">
                <ConfigEditor
                    :value="configuration"
                    title="Application Configuration"
                    description="查看并编辑当前 Application 的完整配置 JSON"
                    :on-submit="handleConfigurationSubmit"
                />
            </div>

            <div v-show="activeTab === 'keys'">
                <section class="rounded-box border border-base-200 bg-base-100">
                    <div class="flex flex-col gap-4 px-5 py-5">
                        <div
                            class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between"
                        >
                            <div class="space-y-1">
                                <h3
                                    class="text-base font-medium text-base-content"
                                >
                                    Signing Keys
                                </h3>
                                <p class="text-sm text-base-content/60">
                                    管理当前 Application 用于签发 JWT
                                    的密钥。轮换密钥会生成一个新的密钥对并使其进入生效周期。
                                </p>
                            </div>

                            <button
                                type="button"
                                class="btn btn-primary btn-sm"
                                :disabled="
                                    loadingKeys || creatingKey || !application
                                "
                                :class="{ loading: creatingKey }"
                                @click="handleRotateKey"
                            >
                                轮换密钥
                            </button>
                        </div>

                        <div
                            v-if="newlyCreatedKey"
                            class="rounded-box border border-success/20 bg-success/5 p-4"
                        >
                            <div class="text-sm font-medium text-base-content">
                                新生成的密钥
                            </div>
                            <div
                                class="mt-2 space-y-1 font-mono text-sm text-base-content"
                            >
                                <div>Key ID: {{ newlyCreatedKey.key_id }}</div>
                                <div>
                                    Algorithm: {{ newlyCreatedKey.algorithm }}
                                </div>
                            </div>
                        </div>

                        <ApplicationKeyTable
                            :keys="keys"
                            :loading="loadingKeys"
                            :revoking-key-id="revokingKeyId"
                            @rotate="handleRotateKey"
                            @revoke="handleRevokeKey"
                        />
                    </div>
                </section>
            </div>

            <div v-show="activeTab === 'secrets'">
                <section class="rounded-box border border-base-200 bg-base-100">
                    <div class="flex flex-col gap-4 px-5 py-5">
                        <div class="space-y-1">
                            <h3 class="text-base font-medium text-base-content">
                                API 密钥
                            </h3>
                            <p class="text-sm text-base-content/60">
                                当前 Application 绑定的 API 密钥列表。
                            </p>
                        </div>

                        <div v-if="loadingSecrets" class="space-y-3">
                            <div class="skeleton h-12 w-full"></div>
                            <div class="skeleton h-12 w-full"></div>
                        </div>

                        <div
                            v-else-if="secrets.length === 0"
                            class="text-sm text-base-content/60"
                        >
                            暂未绑定 API 密钥。
                        </div>

                        <div v-else class="overflow-x-auto">
                            <table class="table table-zebra min-w-220">
                                <thead>
                                    <tr class="text-sm text-base-content/70">
                                        <th>Secret ID</th>
                                        <th>Secret</th>
                                        <th>创建时间</th>
                                        <th>状态</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <tr
                                        v-for="secret in secrets"
                                        :key="secret.id"
                                        class="hover"
                                    >
                                        <td class="font-mono text-xs">
                                            {{ secret.id }}
                                        </td>
                                        <td class="font-mono text-xs">
                                            {{ secret.secret }}
                                        </td>
                                        <td class="text-sm">
                                            {{ secret.created_at }}
                                        </td>
                                        <td>
                                            <span
                                                class="badge badge-sm"
                                                :class="
                                                    secret.revoked_at
                                                        ? 'badge-error badge-soft'
                                                        : 'badge-success badge-soft'
                                                "
                                            >
                                                {{
                                                    secret.revoked_at
                                                        ? "已撤销"
                                                        : "有效"
                                                }}
                                            </span>
                                        </td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>
                    </div>
                </section>
            </div>
        </div>
    </EntityListPage>
</template>
