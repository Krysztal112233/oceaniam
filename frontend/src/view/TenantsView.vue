<script setup lang="ts">
import { OceanIamClient } from "@oceaniam/sdk";
import { computed, onMounted, ref } from "vue";
import EntityListPage from "../components/EntityListPage.vue";
import TenantTable from "../components/TenantTable.vue";
import { appConfig } from "../config";
import { useAuthStore } from "../stores/auth";
import type { TenantVO } from "../../packages/sdk/src/types/TenantVO";

const authStore = useAuthStore();
const client = new OceanIamClient({
    baseUrl: appConfig.systemBaseUrl,
    tokenGetter: () => authStore.jwt,
});

const tenants = ref<TenantVO[]>([]);
const total = ref(0);
const loading = ref(false);
const error = ref<string | null>(null);

const summaryText = computed(() => {
    if (loading.value) {
        return "正在加载 tenant 列表...";
    }

    const count = total.value || tenants.value.length;
    return `共 ${count} 个 tenant`;
});

async function loadTenants() {
    loading.value = true;
    error.value = null;

    try {
        const response = await client.getTenants({
            page: 1n,
            per_page: 50n,
        });

        tenants.value = response.items;
        total.value = response.page_info.total;
    } catch (err) {
        tenants.value = [];
        total.value = 0;
        error.value =
            err instanceof Error ? err.message : "加载 tenant 列表失败。";
    } finally {
        loading.value = false;
    }
}

function handleDetail(tenantId: string) {
    console.debug("tenant detail clicked:", tenantId);
}

function handleDelete(tenantId: string) {
    console.debug("tenant delete clicked:", tenantId);
}

onMounted(() => {
    void loadTenants();
});
</script>

<template>
    <EntityListPage
        page-title="Tenants"
        page-description="管理系统中的 tenant 列表，展示租户标识、备注信息，以及常用操作入口。"
        card-title="Tenant List"
        :summary-text="summaryText"
    >
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

        <div
            v-else-if="tenants.length === 0"
            class="px-6 pb-6 text-sm text-base-content/60"
        >
            当前没有可展示的 tenant。
        </div>

        <TenantTable
            v-else
            :tenants="tenants"
            @detail="handleDetail"
            @delete="handleDelete"
        />
    </EntityListPage>
</template>
