<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, onMounted, ref, watch } from "vue";
import { useRoute } from "vue-router";
import ApplicationTable from "../components/ApplicationTable.vue";
import CreateApplicationModal from "../components/CreateApplicationModal.vue";
import EntityListPage from "../components/EntityListPage.vue";
import { useTenantStore } from "../stores/tenant";

const route = useRoute();
const tenantStore = useTenantStore();
const {
    applications,
    applicationsError,
    applicationsLoading,
    applicationsTotal,
    createApplicationError,
    createApplicationLoading,
    currentTenantId,
    hasTenants,
    tenantsLoading,
} = storeToRefs(tenantStore);
const isCreateDialogOpen = ref(false);
const createSuccessMessage = ref("");

const tenantId = computed(() => {
    const raw = route.params.tenantId;
    return typeof raw === "string" ? raw : "";
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

function handleDetail(applicationId: string) {
    console.debug("application detail clicked:", applicationId);
}

function handleDelete(applicationId: string) {
    console.debug("application delete clicked:", applicationId);
}

function openCreateDialog() {
    createSuccessMessage.value = "";
    isCreateDialogOpen.value = true;
}

function closeCreateDialog() {
    isCreateDialogOpen.value = false;
}

async function handleCreateApplication(comment: string) {
    try {
        await tenantStore.createApplication(comment);
        createSuccessMessage.value = "Application 已创建，列表已刷新。";
        closeCreateDialog();
    } catch {
        // error is stored in tenant store
    }
}

watch(
    () => tenantId.value,
    (nextTenantId) => {
        tenantStore.syncCurrentTenant(nextTenantId);
        void tenantStore.loadApplications(nextTenantId);
    },
);

onMounted(() => {
    tenantStore.syncCurrentTenant(tenantId.value);
    void tenantStore.loadApplications(tenantId.value);
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

        <div v-if="createSuccessMessage" class="px-6 pt-6">
            <div class="alert alert-success alert-soft">
                <span>{{ createSuccessMessage }}</span>
            </div>
        </div>

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

        <ApplicationTable
            v-else
            :applications="applications"
            @detail="handleDetail"
            @delete="handleDelete"
        />
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
