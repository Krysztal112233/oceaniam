<script setup lang="ts">
import { computed } from "vue";

type AuditInfo = {
    id: string;
    audit_type: string;
    payload: unknown;
    created_at: string;
};

const props = defineProps<{
    open: boolean;
    audit: AuditInfo | null;
}>();

const emit = defineEmits<{
    (event: "close"): void;
}>();

const formattedPayload = computed(() => {
    if (!props.audit?.payload) return "";
    try {
        return JSON.stringify(props.audit.payload, null, 2);
    } catch {
        return String(props.audit.payload);
    }
});

const formattedTime = computed(() => {
    if (!props.audit?.created_at) return "";
    try {
        return new Date(props.audit.created_at).toLocaleString("zh-CN", {
            year: "numeric",
            month: "2-digit",
            day: "2-digit",
            hour: "2-digit",
            minute: "2-digit",
            second: "2-digit",
        });
    } catch {
        return props.audit.created_at;
    }
});

function close() {
    emit("close");
}

function onOverlayClick() {
    close();
}
</script>

<template>
    <Teleport to="body">
        <div v-if="open && audit" class="fixed inset-0 z-50 flex justify-end">
            <div
                class="fixed inset-0 bg-black/40 transition-opacity"
                @click="onOverlayClick"
            ></div>

            <div
                class="relative flex w-full max-w-lg flex-col bg-base-100 shadow-xl transition-transform duration-300"
            >
                <div
                    class="flex items-center justify-between border-b border-base-200 px-6 py-4"
                >
                    <div class="min-w-0 space-y-1">
                        <h3 class="text-lg font-medium text-base-content">
                            Audit Detail
                        </h3>
                        <p class="truncate text-sm text-base-content/60">
                            {{ audit.audit_type }}
                        </p>
                    </div>
                    <button
                        type="button"
                        class="btn btn-ghost btn-sm btn-square"
                        aria-label="Close"
                        @click="close"
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            class="h-5 w-5"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                d="M6 18L18 6M6 6l12 12"
                            />
                        </svg>
                    </button>
                </div>

                <div class="flex-1 overflow-y-auto px-6 py-4">
                    <div class="space-y-4">
                        <div
                            class="rounded-box border border-base-200 bg-base-100"
                        >
                            <div class="divide-y divide-base-200">
                                <div class="flex items-start gap-4 px-4 py-3">
                                    <span
                                        class="shrink-0 text-sm font-medium text-base-content/60"
                                    >
                                        ID
                                    </span>
                                    <span
                                        class="break-all font-mono text-sm text-base-content"
                                    >
                                        {{ audit.id }}
                                    </span>
                                </div>
                                <div class="flex items-start gap-4 px-4 py-3">
                                    <span
                                        class="shrink-0 text-sm font-medium text-base-content/60"
                                    >
                                        Type
                                    </span>
                                    <span class="badge badge-soft badge-sm">
                                        {{ audit.audit_type }}
                                    </span>
                                </div>
                                <div class="flex items-start gap-4 px-4 py-3">
                                    <span
                                        class="shrink-0 text-sm font-medium text-base-content/60"
                                    >
                                        Created At
                                    </span>
                                    <span class="text-sm text-base-content">
                                        {{ formattedTime }}
                                    </span>
                                </div>
                            </div>
                        </div>

                        <div>
                            <h4
                                class="mb-2 text-sm font-medium text-base-content/60"
                            >
                                Payload
                            </h4>
                            <pre
                                class="overflow-x-auto rounded-box border border-base-200 bg-base-50 p-4 text-xs leading-relaxed text-base-content"
                            >
                                <code>{{ formattedPayload }}</code></pre>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </Teleport>
</template>
