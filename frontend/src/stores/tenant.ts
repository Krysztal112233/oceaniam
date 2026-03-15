import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { OceanIamClient } from "@oceaniam/sdk";
import { appConfig } from "../config";
import { useAuthStore } from "./auth";
import type { ApplicationVO } from "../../packages/sdk/src/types/ApplicationVO";
import type { TenantVO } from "../../packages/sdk/src/types/TenantVO";

function getClient(): OceanIamClient {
    const authStore = useAuthStore();

    return new OceanIamClient({
        baseUrl: appConfig.systemBaseUrl,
        tokenGetter: () => authStore.jwt,
    });
}

export const useTenantStore = defineStore(
    "tenant",
    () => {
        const tenants = ref<TenantVO[]>([]);
        const tenantsLoading = ref(false);
        const tenantsError = ref<string | null>(null);

        const currentTenantId = ref("");

        const applications = ref<ApplicationVO[]>([]);
        const applicationsTotal = ref(0);
        const applicationsLoading = ref(false);
        const applicationsError = ref<string | null>(null);
        const createApplicationLoading = ref(false);
        const createApplicationError = ref<string | null>(null);

        const hasTenants = computed(() => tenants.value.length > 0);
        const currentTenant = computed(
            () =>
                tenants.value.find(
                    (tenant) => tenant.id === currentTenantId.value,
                ) ?? null,
        );

        function syncCurrentTenant(tenantId: string): void {
            currentTenantId.value = tenantId.trim();
        }

        function ensureCurrentTenant(): string {
            const normalizedCurrentTenantId = currentTenantId.value.trim();
            const matchedTenant = tenants.value.find(
                (tenant) => tenant.id === normalizedCurrentTenantId,
            );

            if (matchedTenant) {
                currentTenantId.value = matchedTenant.id;
                return matchedTenant.id;
            }

            const fallbackTenant = tenants.value[0];
            if (fallbackTenant) {
                currentTenantId.value = fallbackTenant.id;
                return fallbackTenant.id;
            }

            currentTenantId.value = "";
            return "";
        }

        function clearApplicationsState(): void {
            applications.value = [];
            applicationsTotal.value = 0;
            applicationsLoading.value = false;
            applicationsError.value = null;
            createApplicationLoading.value = false;
            createApplicationError.value = null;
        }

        function clearTenantState(): void {
            tenants.value = [];
            tenantsLoading.value = false;
            tenantsError.value = null;
            currentTenantId.value = "";
            clearApplicationsState();
        }

        async function loadTenants(): Promise<void> {
            const authStore = useAuthStore();
            if (!authStore.isLoggedIn) {
                tenants.value = [];
                tenantsError.value = null;
                return;
            }

            const client = getClient();

            tenantsLoading.value = true;
            tenantsError.value = null;

            try {
                const response = await client.getTenants({
                    page: 1n,
                    per_page: 50n,
                });
                tenants.value = response.items;
                ensureCurrentTenant();
            } catch (err) {
                tenants.value = [];
                tenantsError.value =
                    err instanceof Error
                        ? err.message
                        : "加载 tenant 列表失败。";
            } finally {
                tenantsLoading.value = false;
            }
        }

        async function loadApplications(
            tenantId = currentTenantId.value,
        ): Promise<void> {
            const normalizedTenantId = tenantId.trim();
            syncCurrentTenant(normalizedTenantId);

            if (!normalizedTenantId) {
                clearApplicationsState();
                applicationsError.value =
                    "缺少 tenant 标识，无法加载 application 列表。";
                return;
            }

            const client = getClient();

            applicationsLoading.value = true;
            applicationsError.value = null;

            try {
                const response = await client.getApplications({
                    tenant_id: normalizedTenantId,
                    page: 1n,
                    per_page: 50n,
                });
                applications.value = response.items;
                applicationsTotal.value = response.page_info.total;
            } catch (err) {
                applications.value = [];
                applicationsTotal.value = 0;
                applicationsError.value =
                    err instanceof Error
                        ? err.message
                        : "加载 application 列表失败。";
            } finally {
                applicationsLoading.value = false;
            }
        }

        async function createApplication(comment: string): Promise<void> {
            const normalizedTenantId = currentTenantId.value.trim();

            if (!normalizedTenantId) {
                createApplicationError.value =
                    "缺少 tenant 标识，无法创建 application。";
                return;
            }

            const client = getClient();

            createApplicationLoading.value = true;
            createApplicationError.value = null;

            try {
                await client.createApplication({
                    tenant_id: normalizedTenantId,
                    comment: comment.trim() || null,
                });
                await loadApplications(normalizedTenantId);
            } catch (err) {
                createApplicationError.value =
                    err instanceof Error
                        ? err.message
                        : "创建 application 失败。";
                throw err;
            } finally {
                createApplicationLoading.value = false;
            }
        }

        return {
            tenants,
            tenantsLoading,
            tenantsError,
            hasTenants,
            currentTenantId,
            currentTenant,
            applications,
            applicationsTotal,
            applicationsLoading,
            applicationsError,
            createApplicationLoading,
            createApplicationError,
            syncCurrentTenant,
            ensureCurrentTenant,
            clearApplicationsState,
            clearTenantState,
            loadTenants,
            loadApplications,
            createApplication,
        };
    },
    {
        persist: {
            pick: ["currentTenantId"],
        },
    },
);
