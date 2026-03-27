<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useToast } from "vue-toastification";
import ApplicationTable from "../components/ApplicationTable.vue";
import CreateApplicationModal from "../components/CreateApplicationModal.vue";
import EntityListPage from "../components/EntityListPage.vue";
import { useTenantStore } from "../stores/tenant";

const route = useRoute();
const router = useRouter();
const tenantStore = useTenantStore();
const toast = useToast();
const {
    applications,
    applicationsError,
    applicationsLoading,
    applicationsTotal,
    createApplicationError,
    createApplicationLoading,
    currentTenantId,
    deleteApplicationError,
    deleteApplicationLoading,
    hasTenants,
    tenantsLoading,
} = storeToRefs(tenantStore);
const isCreateDialogOpen = ref(false);
const deletingApplicationId = ref("");
const currentPage = ref(1);
const perPage = 25;
const hasNextPage = ref(false);

const tenantId = computed(() => {
    const raw = route.params.tenantId;
    return typeof raw === "string" ? raw : "";
});

const totalPages = computed(() =>
    applicationsTotal.value === 0
        ? 1
        : Math.ceil(applicationsTotal.value / perPage),
);

const canGoToPreviousPage = computed(() => currentPage.value > 1);

const canGoToNextPage = computed(
    () => hasNextPage.value && currentPage.value < totalPages.value,
);

const paginationSummary = computed(() => {
    if (applicationsTotal.value === 0 || applications.value.length === 0) {
        return "第 0 - 0 条，共 0 条";
    }

    const start = (currentPage.value - 1) * perPage + 1;
    const end = start + applications.value.length - 1;
    return `第 ${start} - ${end} 条，共 ${applicationsTotal.value} 条`;
});

const summaryText = computed(() => {
    const activeTenantId = tenantId.value || currentTenantId.value;

    if (tenantsLoading.value) {
        return "正在恢复 tenant 上下文...";
    }

    if (!activeTenantId) {
        return hasTenants.value ? "请先选择 tenant。" : "当前没有可用 tenant。";
    }

    if (applicationsLoading.value) {
        return `正在加载 ${activeTenantId} 下的 application 列表...`;
    }

    const count = applicationsTotal.value || applications.value.length;
    return `租户 ${activeTenantId} 下共 ${count} 个 application`;
});

async function handleDetail(applicationId: string) {
    await router.push({
        name: "application-detail",
        params: {
            tenantId: tenantId.value || currentTenantId.value,
            applicationId,
        },
    });
}

function handleDelete(applicationId: string) {
    deletingApplicationId.value = applicationId;

    void confirmDeleteApplication(applicationId);
}

function openCreateDialog() {
    isCreateDialogOpen.value = true;
}

function closeCreateDialog() {
    isCreateDialogOpen.value = false;
}

async function handleCreateApplication(comment: string) {
    try {
        currentPage.value = 1;
        await tenantStore.createApplication(comment, {
            page: currentPage.value,
            per_page: perPage,
        });
        hasNextPage.value = applicationsTotal.value > perPage;
        closeCreateDialog();
        toast.success("Application 已创建");
    } catch {
        toast.error(createApplicationError.value || "创建 Application 失败。");
    }
}

async function confirmDeleteApplication(applicationId: string) {
    try {
        await tenantStore.deleteApplication(applicationId, {
            page: currentPage.value,
            per_page: perPage,
        });
        hasNextPage.value =
            currentPage.value * perPage < applicationsTotal.value;
        deletingApplicationId.value = "";
        toast.success("Application 已删除");
    } catch {
        deletingApplicationId.value = applicationId;
        toast.error(deleteApplicationError.value || "删除 Application 失败。");
    }
}

async function loadApplicationsPage(nextTenantId: string): Promise<void> {
    const response = await tenantStore.loadApplications(nextTenantId, {
        page: currentPage.value,
        per_page: perPage,
    });
    hasNextPage.value = response?.page_info.has_next ?? false;
}

async function goToPreviousPage(): Promise<void> {
    if (!canGoToPreviousPage.value || applicationsLoading.value) {
        return;
    }

    currentPage.value -= 1;
    await loadApplicationsPage(tenantId.value || currentTenantId.value);
}

async function goToNextPage(): Promise<void> {
    if (!canGoToNextPage.value || applicationsLoading.value) {
        return;
    }

    currentPage.value += 1;
    await loadApplicationsPage(tenantId.value || currentTenantId.value);
}

watch(
    () => tenantId.value,
    (nextTenantId, previousTenantId) => {
        deletingApplicationId.value = "";
        tenantStore.syncCurrentTenant(nextTenantId);

        if (nextTenantId !== previousTenantId) {
            currentPage.value = 1;
        }

        void loadApplicationsPage(nextTenantId);
    },
);

onMounted(() => {
    tenantStore.syncCurrentTenant(tenantId.value);
    void loadApplicationsPage(tenantId.value);
});
</script>

<template>
    <EntityListPage
        page-title="Applications"
        page-description="管理当前租户下的 application 列表，展示应用标识、备注信息，以及常用操作入口。"
        card-title="Application List"
        :summary-text="summaryText"
    >
        <template #actions>
            <button
                type="button"
                class="btn btn-primary btn-sm"
                :disabled="
                    tenantsLoading ||
                    applicationsLoading ||
                    !currentTenantId ||
                    !hasTenants
                "
                @click="openCreateDialog"
            >
                新增 Application
            </button>
        </template>

        <div
            v-if="tenantsLoading || applicationsLoading"
            class="space-y-3 px-6 pb-6"
        >
            <div class="skeleton h-12 w-full"></div>
            <div class="skeleton h-12 w-full"></div>
            <div class="skeleton h-12 w-full"></div>
        </div>

        <div v-else-if="applicationsError" class="px-6 pb-6">
            <div class="alert alert-error alert-soft">
                <span>{{ applicationsError }}</span>
            </div>
        </div>

        <div
            v-else-if="!hasTenants"
            class="px-6 pb-6 text-sm text-base-content/60"
        >
            当前没有可用 tenant。
        </div>

        <div
            v-else-if="applications.length === 0"
            class="px-6 pb-6 text-sm text-base-content/60"
        >
            当前 tenant 下没有可展示的 application。
        </div>

        <div v-else class="px-6 pb-6">
            <div class="space-y-4">
                <ApplicationTable
                    :applications="applications"
                    :deleting-application-id="deletingApplicationId"
                    :delete-loading="deleteApplicationLoading"
                    :delete-error="deleteApplicationError"
                    :pagination-summary="paginationSummary"
                    :current-page="currentPage"
                    :total-pages="totalPages"
                    :can-go-to-previous-page="canGoToPreviousPage"
                    :can-go-to-next-page="canGoToNextPage"
                    :loading="applicationsLoading"
                    @detail="handleDetail"
                    @delete="handleDelete"
                    @previous-page="goToPreviousPage"
                    @next-page="goToNextPage"
                />
            </div>
        </div>
    </EntityListPage>

    <CreateApplicationModal
        :open="isCreateDialogOpen"
        :tenant-id="currentTenantId || tenantId"
        :loading="createApplicationLoading"
        :error="createApplicationError"
        @close="closeCreateDialog"
        @submit="handleCreateApplication"
    />
</template>
