<script setup lang="ts">
import type { ApplicationVO } from "../../packages/sdk/src/types/ApplicationVO";
import ApplicationRowActions from "./ApplicationRowActions.vue";

defineProps<{
    applications: ApplicationVO[];
    deletingApplicationId: string;
    deleteLoading: boolean;
    deleteError: string | null;
}>();

const emit = defineEmits<{
    (event: "detail", applicationId: string): void;
    (event: "delete", applicationId: string): void;
}>();

function formatComment(comment: ApplicationVO["comment"]): string {
    if (!comment) {
        return "暂无备注";
    }

    return comment;
}
</script>

<template>
    <div class="overflow-x-auto">
        <table class="table table-zebra min-w-190">
            <thead>
                <tr class="text-sm text-base-content/70">
                    <th class="whitespace-nowrap">ApplicationID</th>
                    <th>Comment</th>
                    <th class="w-44 whitespace-nowrap text-center">Actions</th>
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
                            @delete="emit('delete', $event)"
                        />
                    </td>
                </tr>
            </tbody>
        </table>
    </div>
</template>
