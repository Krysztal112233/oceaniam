<script setup lang="ts">
import type { ApplicationVO } from "@oceaniam/sdk";
import ApplicationRowActions from "../actions/ApplicationRowActions.vue";

defineProps<{
    applications: ApplicationVO[];
    deletingApplicationId: string;
    deleteLoading: boolean;
    deleteError: string | null;
    paginationSummary: string;
    currentPage: number;
    totalPages: number;
    canGoToPreviousPage: boolean;
    canGoToNextPage: boolean;
    loading?: boolean;
}>();

const emit = defineEmits<{
    (event: "detail", applicationId: string): void;
    (event: "users", applicationId: string): void;
    (event: "delete", applicationId: string): void;
    (event: "previousPage"): void;
    (event: "nextPage"): void;
}>();

function formatComment(comment: ApplicationVO["comment"]): string {
    if (!comment) {
        return "暂无备注";
    }

    return comment;
}
</script>

<template>
    <div>
        <div class="overflow-x-auto">
            <table class="table table-zebra min-w-220">
                <thead>
                    <tr class="text-sm text-base-content/70">
                        <th class="whitespace-nowrap">ApplicationID</th>
                        <th>Comment</th>
                        <th class="w-72 whitespace-nowrap text-center">
                            Actions
                        </th>
                    </tr>
                </thead>
                <tbody>
                    <tr
                        v-for="application in applications"
                        :key="application.id"
                        class="hover"
                    >
                        <td>
                            <div class="font-mono text-sm text-base-content">
                                {{ application.id }}
                            </div>
                        </td>
                        <td>
                            <span
                                class="text-sm"
                                :class="
                                    application.comment
                                        ? 'text-base-content'
                                        : 'text-base-content/50'
                                "
                            >
                                {{ formatComment(application.comment) }}
                            </span>
                        </td>
                        <td>
                            <ApplicationRowActions
                                :application-id="application.id"
                                :deleting="
                                    deleteLoading &&
                                    deletingApplicationId === application.id
                                "
                                :delete-error="
                                    deletingApplicationId === application.id
                                        ? deleteError
                                        : null
                                "
                                @detail="emit('detail', $event)"
                                @users="emit('users', $event)"
                                @delete="emit('delete', $event)"
                            />
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
