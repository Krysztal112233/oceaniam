<script setup lang="ts">
import { computed, defineAsyncComponent, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { OceanIamClient } from "@oceaniam/sdk";
import { useToast } from "vue-toastification";
import type { ApplicationVO } from "../../packages/sdk/src/types/ApplicationVO";
import type { ApplicationConfigurationVO } from "../../packages/sdk/src/types/ApplicationConfigurationVO";
import EntityListPage from "../components/EntityListPage.vue";
import { appConfig } from "../config";
import { useAuthStore } from "../stores/auth";
import { useTenantStore } from "../stores/tenant";

const ConfigEditor = defineAsyncComponent(
    () => import("../components/ConfigEditor.vue"),
);

const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();
const tenantStore = useTenantStore();
const toast = useToast();

const application = ref<ApplicationVO | null>(null);
const configuration = ref<ApplicationConfigurationVO | null>(null);
const commentDraft = ref("");
const commentSaving = ref(false);
const loading = ref(false);
const error = ref<string | null>(null);
const requestId = ref(0);

const tenantId = computed(() => {
    const raw = route.params.tenantId;
    return typeof raw === "string" ? raw : "";
});

const applicationId = computed(() => {
    const raw = route.params.applicationId;
    return typeof raw === "string" ? raw : "";
});

function getClient(): OceanIamClient {
    return new OceanIamClient({
        baseUrl: appConfig.systemBaseUrl,
        tokenGetter: () => authStore.jwt,
    });
}

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

const commentSaveEnabled = computed(() => {
    const currentComment = application.value?.comment ?? "";
    return !commentSaving.value && commentDraft.value !== currentComment;
});

async function loadApplicationDetail(): Promise<void> {
    const normalizedTenantId = tenantId.value.trim();
    const normalizedApplicationId = applicationId.value.trim();

    application.value = null;
    configuration.value = null;
    error.value = null;

    if (!normalizedTenantId || !normalizedApplicationId) {
        error.value = "缺少 tenant 或 application 标识。";
        return;
    }

    tenantStore.syncCurrentTenant(normalizedTenantId);

    const currentRequestId = requestId.value + 1;
    requestId.value = currentRequestId;
    loading.value = true;

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
                authentication: {
                    issuer: nextConfiguration.authentication.issuer,
                    audience: nextConfiguration.authentication.audience,
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
        page-description="展示当前 application 的基础信息与完整配置 JSON。"
        card-title="Application Information"
        :summary-text="summaryText"
    >
        <template #actions>
            <button
                type="button"
                class="btn btn-outline btn-sm"
                @click="goBack"
            >
                返回 Applications
            </button>
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
            </section>

            <ConfigEditor
                :value="configuration"
                title="Application Configuration"
                description="查看并编辑当前 Application 的完整配置 JSON"
                :on-submit="handleConfigurationSubmit"
            />

            <section></section>
        </div>
    </EntityListPage>
</template>
