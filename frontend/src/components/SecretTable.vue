<script setup lang="ts">
import { computed } from "vue";
import type { SecretVO } from "../../packages/sdk/src/types/SecretVO";

const props = defineProps<{
    secrets: SecretVO[];
    applicationComments: Record<string, string>;
    paginationSummary: string;
    currentPage: number;
    totalPages: number;
    canGoToPreviousPage: boolean;
    canGoToNextPage: boolean;
    loading?: boolean;
}>();

const emit = defineEmits<{
    previousPage: [];
    nextPage: [];
}>();

const rows = computed(() =>
    props.secrets.map((secret) => ({
        ...secret,
        maskedSecret: formatSecret(secret.secret),
        createdAt: formatDateTime(secret.created_at),
        revokedAt: secret.revoked_at
            ? formatDateTime(secret.revoked_at)
            : "有效",
    })),
);

function formatSecret(secret: string): string {
    if (secret.length <= 12) {
        return secret;
    }

    return `${secret.slice(0, 6)}...${secret.slice(-4)}`;
}

function formatDateTime(value: string): string {
    const date = new Date(value);

    if (Number.isNaN(date.getTime())) {
        return value;
    }

    return date.toLocaleString("zh-CN", {
        hour12: false,
    });
}

function formatApplicationIds(applicationIds: string[]): string[] {
    if (applicationIds.length === 0) {
        return ["未关联应用"];
    }

    return applicationIds.map((applicationId) => {
        const comment = props.applicationComments[applicationId];
        return comment ? `${applicationId} (${comment})` : applicationId;
    });
}
</script>

<template>
    <div>
        <div class="overflow-x-auto">
            <table class="table table-zebra min-w-220">
                <thead>
                    <tr class="text-sm text-base-content/70">
                        <th class="whitespace-nowrap">Secret ID</th>
                        <th class="whitespace-nowrap">Secret</th>
                        <th class="whitespace-nowrap">Created At</th>
                        <th class="whitespace-nowrap">Status</th>
                        <th>Applications</th>
                    </tr>
                </thead>
                <tbody>
                    <tr v-for="row in rows" :key="row.id" class="hover">
                        <td>
                            <div class="font-mono text-sm text-base-content">
                                {{ row.id }}
                            </div>
                        </td>
                        <td>
                            <div
                                class="max-w-48 truncate font-mono text-sm text-base-content"
                                :title="row.secret"
                            >
                                {{ row.maskedSecret }}
                            </div>
                        </td>
                        <td class="text-sm text-base-content/80">
                            {{ row.createdAt }}
                        </td>
                        <td>
                            <span
                                class="badge badge-sm"
                                :class="
                                    row.revoked_at
                                        ? 'badge-error badge-soft'
                                        : 'badge-success badge-soft'
                                "
                            >
                                {{ row.revoked_at ? "已撤销" : "有效" }}
                            </span>
                            <div
                                v-if="row.revoked_at"
                                class="mt-1 text-xs text-base-content/60"
                            >
                                {{ row.revokedAt }}
                            </div>
                        </td>
                        <td>
                            <div class="flex flex-wrap gap-2">
                                <span
                                    v-for="applicationId in formatApplicationIds(
                                        row.application_ids,
                                    )"
                                    :key="applicationId"
                                    class="badge badge-outline"
                                >
                                    {{ applicationId }}
                                </span>
                            </div>
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
