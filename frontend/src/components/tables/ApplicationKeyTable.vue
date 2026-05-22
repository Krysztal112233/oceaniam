<script setup lang="ts">
import type { ApplicationKeyVO } from "@oceaniam/sdk";

defineProps<{
    keys: ApplicationKeyVO[];
    loading: boolean;
    revokingKeyId: string | null;
}>();

const emit = defineEmits<{
    rotate: [];
    revoke: [keyId: string];
}>();

function formatDateTime(value: string | null): string {
    if (!value) return "—";
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return value;
    return date.toLocaleString("zh-CN", { hour12: false });
}

function statusBadgeClass(status: string): string {
    switch (status) {
        case "Active":
            return "badge-success badge-soft";
        case "Revoked":
            return "badge-error badge-soft";
        default:
            return "badge-ghost badge-soft";
    }
}
</script>

<template>
    <div>
        <div v-if="loading" class="space-y-3">
            <div class="skeleton h-12 w-full"></div>
            <div class="skeleton h-12 w-full"></div>
        </div>

        <div v-else-if="keys.length === 0" class="text-sm text-base-content/60">
            当前没有可展示的密钥。
        </div>

        <div v-else class="overflow-x-auto">
            <table class="table table-zebra min-w-220">
                <thead>
                    <tr class="text-sm text-base-content/70">
                        <th class="whitespace-nowrap">Key ID</th>
                        <th class="whitespace-nowrap">Algorithm</th>
                        <th class="whitespace-nowrap">Status</th>
                        <th class="whitespace-nowrap">Created At</th>
                        <th class="whitespace-nowrap">Activated At</th>
                        <th class="whitespace-nowrap">Expires At</th>
                        <th class="whitespace-nowrap">Actions</th>
                    </tr>
                </thead>
                <tbody>
                    <tr v-for="key in keys" :key="key.key_id" class="hover">
                        <td>
                            <div
                                class="max-w-56 break-all font-mono text-sm text-base-content"
                            >
                                {{ key.key_id }}
                            </div>
                        </td>
                        <td class="text-sm text-base-content/80">
                            {{ key.algorithm }}
                        </td>
                        <td>
                            <span
                                class="badge badge-sm"
                                :class="statusBadgeClass(key.status)"
                            >
                                {{ key.status }}
                            </span>
                        </td>
                        <td class="text-sm text-base-content/80">
                            {{ formatDateTime(key.created_at) }}
                        </td>
                        <td class="text-sm text-base-content/80">
                            {{ formatDateTime(key.activated_at) }}
                        </td>
                        <td class="text-sm text-base-content/80">
                            {{ formatDateTime(key.expires_at) }}
                        </td>
                        <td>
                            <button
                                v-if="key.status === 'Active'"
                                type="button"
                                class="btn btn-error btn-outline btn-xs"
                                :disabled="revokingKeyId === key.key_id"
                                :class="{
                                    loading: revokingKeyId === key.key_id,
                                }"
                                @click="emit('revoke', key.key_id)"
                            >
                                吊销
                            </button>
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>
    </div>
</template>
