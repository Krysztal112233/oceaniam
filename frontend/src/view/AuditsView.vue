<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRoute } from "vue-router";
import AuditPayloadDrawer from "../components/AuditPayloadDrawer.vue";
import AuditTable from "../components/AuditTable.vue";
import EntityListPage from "../components/EntityListPage.vue";
import { getClient } from "../utils/api-client";

type AuditLogItem = {
    id: string;
    audit_type: string;
    payload: unknown;
    created_at: string;
};

const route = useRoute();

const items = ref<AuditLogItem[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);
const requestId = ref(0);
const currentPage = ref(1);
const perPage = 25;
const totalItems = ref(0);
const hasNextPage = ref(false);
const auditTypes = ref<string[]>([]);
const selectedAuditType = ref("");

const drawerAudit = ref<AuditLogItem | null>(null);
const drawerOpen = ref(false);

const totalPages = computed(() =>
    totalItems.value === 0 ? 1 : Math.ceil(totalItems.value / perPage),
);

const canGoToPreviousPage = computed(() => currentPage.value > 1);

const canGoToNextPage = computed(
    () => hasNextPage.value && currentPage.value < totalPages.value,
);

const paginationSummary = computed(() => {
    if (totalItems.value === 0 || items.value.length === 0) {
        return "第 0 - 0 条，共 0 条";
    }
    const start = (currentPage.value - 1) * perPage + 1;
    const end = start + items.value.length - 1;
    return `第 ${start} - ${end} 条，共 ${totalItems.value} 条`;
});

function extractAuditTypes(data: AuditLogItem[]): void {
    const unique = new Set<string>();
    for (const item of data) {
        if (item.audit_type) unique.add(item.audit_type);
    }
    const currentSet = new Set(auditTypes.value);
    const newSet = new Set([...currentSet, ...unique]);
    if (newSet.size !== currentSet.size) {
        auditTypes.value = [...newSet].sort();
    }
}

function resetPageState(): void {
    items.value = [];
    loading.value = false;
    error.value = null;
    currentPage.value = 1;
    totalItems.value = 0;
    hasNextPage.value = false;
}

async function loadAudits(): Promise<void> {
    const rid = requestId.value + 1;
    requestId.value = rid;
    loading.value = true;
    error.value = null;

    try {
        const query: { page: number; per_page: number; audit_type?: string } = {
            page: currentPage.value,
            per_page: perPage,
        };
        if (selectedAuditType.value) {
            query.audit_type = selectedAuditType.value;
        }

        const response = await getClient().getAudits(query);

        if (rid !== requestId.value) return;

        items.value = response.items.map((item) => ({
            id: item.id,
            audit_type: item.audit_type,
            payload: item.payload as unknown,
            created_at: item.created_at,
        }));
        totalItems.value = response.page_info.total;
        hasNextPage.value = response.page_info.has_next;
        extractAuditTypes(items.value);
    } catch (err) {
        if (rid !== requestId.value) return;

        items.value = [];
        totalItems.value = 0;
        hasNextPage.value = false;
        error.value = err instanceof Error ? err.message : "加载审计日志失败。";
    } finally {
        if (rid === requestId.value) {
            loading.value = false;
        }
    }
}

function onFilterChange(): void {
    currentPage.value = 1;
    void loadAudits();
}

async function goToPreviousPage(): Promise<void> {
    if (!canGoToPreviousPage.value || loading.value) return;
    currentPage.value -= 1;
    await loadAudits();
}

async function goToNextPage(): Promise<void> {
    if (!canGoToNextPage.value || loading.value) return;
    currentPage.value += 1;
    await loadAudits();
}

function openPayloadDrawer(item: AuditLogItem): void {
    drawerAudit.value = item;
    drawerOpen.value = true;
}

function closePayloadDrawer(): void {
    drawerOpen.value = false;
    drawerAudit.value = null;
}

onMounted(() => {
    void loadAudits();
});

watch(
    () => route.path,
    () => {
        resetPageState();
        void loadAudits();
    },
);
</script>

<template>
    <EntityListPage
        page-title="Audits"
        page-description="查看平台关键操作的审计日志记录，支持按类型筛选和 JSON Payload 详情查看。"
        card-title="Audit Logs"
        :summary-text="paginationSummary"
    >
        <template #actions>
            <div class="flex items-center gap-3">
                <label class="text-sm text-base-content/60">Type</label>
                <select
                    v-model="selectedAuditType"
                    class="select select-bordered select-sm"
                    :disabled="loading"
                    @change="onFilterChange"
                >
                    <option value="">All</option>
                    <option
                        v-for="type in auditTypes"
                        :key="type"
                        :value="type"
                    >
                        {{ type }}
                    </option>
                </select>
            </div>
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

        <div
            v-else-if="items.length === 0"
            class="px-6 pb-6 text-sm text-base-content/60"
        >
            当前没有可展示的审计日志。
        </div>

        <div v-else class="px-6 pb-6">
            <AuditTable
                :items="items"
                :loading="loading"
                :pagination-summary="paginationSummary"
                :current-page="currentPage"
                :total-pages="totalPages"
                :can-go-to-previous-page="canGoToPreviousPage"
                :can-go-to-next-page="canGoToNextPage"
                @view-payload="openPayloadDrawer"
                @previous-page="goToPreviousPage"
                @next-page="goToNextPage"
            />
        </div>
    </EntityListPage>

    <AuditPayloadDrawer
        :open="drawerOpen"
        :audit="drawerAudit"
        @close="closePayloadDrawer"
    />
</template>
