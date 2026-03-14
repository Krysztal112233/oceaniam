<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, onMounted, watch } from "vue";
import { useRoute } from "vue-router";
import ApplicationTable from "../components/ApplicationTable.vue";
import EntityListPage from "../components/EntityListPage.vue";
import { useTenantStore } from "../stores/tenant";

const route = useRoute();
const tenantStore = useTenantStore();
const {
    applications,
    applicationsError,
    applicationsLoading,
    applicationsTotal,
    currentTenantId,
    hasTenants,
    tenantsLoading,
} = storeToRefs(tenantStore);

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
        return hasTenants.value
            ? "请先选择 tenant。"
            : "当前没有可用 tenant。";
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
        <div v-if="tenantsLoading || applicationsLoading" class="space-y-3 px-6 pb-6">
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
</template>
