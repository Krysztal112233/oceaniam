<script setup lang="ts">
type AuditLogItem = {
    id: string;
    audit_type: string;
    payload: unknown;
    created_at: string;
};

defineProps<{
    items: AuditLogItem[];
    loading: boolean;
    paginationSummary: string;
    currentPage: number;
    totalPages: number;
    canGoToPreviousPage: boolean;
    canGoToNextPage: boolean;
}>();

const emit = defineEmits<{
    (event: "viewPayload", item: AuditLogItem): void;
    (event: "previousPage"): void;
    (event: "nextPage"): void;
}>();

function formatPayload(payload: unknown): string {
    try {
        return JSON.stringify(payload);
    } catch {
        return String(payload);
    }
}

function formatTimestamp(iso: string): string {
    try {
        const date = new Date(iso);
        return date.toLocaleString("zh-CN", {
            year: "numeric",
            month: "2-digit",
            day: "2-digit",
            hour: "2-digit",
            minute: "2-digit",
            second: "2-digit",
        });
    } catch {
        return iso;
    }
}
</script>

<template>
    <div>
        <div class="overflow-x-auto">
            <table class="table table-zebra min-w-220">
                <thead>
                    <tr class="text-sm text-base-content/70">
                        <th class="whitespace-nowrap">ID</th>
                        <th class="whitespace-nowrap">Type</th>
                        <th class="whitespace-nowrap">Payload</th>
                        <th class="whitespace-nowrap">Created At</th>
                        <th class="w-24 whitespace-nowrap text-center">
                            Actions
                        </th>
                    </tr>
                </thead>
                <tbody>
                    <tr v-for="item in items" :key="item.id" class="hover">
                        <td>
                            <div
                                class="max-w-40 truncate font-mono text-sm text-base-content"
                            >
                                {{ item.id }}
                            </div>
                        </td>
                        <td>
                            <span class="badge badge-soft badge-sm">
                                {{ item.audit_type }}
                            </span>
                        </td>
                        <td>
                            <span
                                class="block max-w-80 truncate font-mono text-xs text-base-content/60"
                            >
                                {{ formatPayload(item.payload) }}
                            </span>
                        </td>
                        <td class="whitespace-nowrap text-sm">
                            {{ formatTimestamp(item.created_at) }}
                        </td>
                        <td class="text-center">
                            <button
                                type="button"
                                class="btn btn-ghost btn-xs"
                                @click="emit('viewPayload', item)"
                            >
                                查看
                            </button>
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>

        <div
            class="mt-4 flex flex-col gap-3 border-t border-base-200 pt-4 text-sm text-base-content/70 md:flex-row md:items-center md:justify-between"
        >
            <div>{{ paginationSummary }}</div>

            <div class="join">
                <button
                    type="button"
                    class="btn btn-sm join-item"
                    :disabled="!canGoToPreviousPage || loading"
                    @click="emit('previousPage')"
                >
                    上一页
                </button>
                <button type="button" class="btn btn-sm join-item" disabled>
                    第 {{ currentPage }} / {{ totalPages }} 页
                </button>
                <button
                    type="button"
                    class="btn btn-sm join-item"
                    :disabled="!canGoToNextPage || loading"
                    @click="emit('nextPage')"
                >
                    下一页
                </button>
            </div>
        </div>
    </div>
</template>
