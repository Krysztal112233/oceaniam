<script setup lang="ts">
import type { TenantVO } from "@oceaniam/sdk";
import TenantRowActions from "./TenantRowActions.vue";

defineProps<{
    tenants: TenantVO[];
}>();

const emit = defineEmits<{
    (event: "detail", tenantId: string): void;
    (event: "delete", tenantId: string): void;
}>();

function formatComment(comment: TenantVO["comment"]): string {
    if (!comment) {
        return "暂无备注";
    }

    return comment;
}
</script>

<template>
    <div class="overflow-x-auto">
        <table class="table table-zebra min-w-[760px]">
            <thead>
                <tr class="text-sm text-base-content/70">
                    <th class="whitespace-nowrap">TenantID</th>
                    <th>Comment</th>
                    <th class="w-44 whitespace-nowrap text-right">Actions</th>
                </tr>
            </thead>
            <tbody>
                <tr v-for="tenant in tenants" :key="tenant.id" class="hover">
                    <td class="align-top">
                        <div class="font-mono text-sm text-base-content">
                            {{ tenant.id }}
                        </div>
                    </td>
                    <td class="align-top">
                        <span
                            class="text-sm"
                            :class="
                                tenant.comment
                                    ? 'text-base-content'
                                    : 'text-base-content/50'
                            "
                        >
                            {{ formatComment(tenant.comment) }}
                        </span>
                    </td>
                    <td class="align-top">
                        <TenantRowActions
                            :tenant-id="tenant.id"
                            @detail="emit('detail', $event)"
                            @delete="emit('delete', $event)"
                        />
                    </td>
                </tr>
            </tbody>
        </table>
    </div>
</template>
