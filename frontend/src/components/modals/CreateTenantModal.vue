<script setup lang="ts">
import CloseIcon from "@iconify-vue/material-symbols/close-rounded";
import { ref, watch } from "vue";
import { useToast } from "vue-toastification";
import { useTenantStore } from "../../stores/tenant";
import { getClient } from "../../utils/api-client";

const props = defineProps<{
    open: boolean;
}>();

const emit = defineEmits<{
    (event: "close"): void;
    (event: "created", tenantId: string): void;
}>();

const tenantStore = useTenantStore();
const toast = useToast();
const dialogRef = ref<HTMLDialogElement | null>(null);
const comment = ref("");
const loading = ref(false);
const error = ref<string | null>(null);

watch(
    () => props.open,
    (open) => {
        const dialog = dialogRef.value;
        if (!dialog) return;

        if (open) {
            comment.value = "";
            error.value = null;
            try {
                if (!dialog.open) dialog.showModal();
            } catch {
                // ignore
            }
            return;
        }

        if (dialog.open) dialog.close();
    },
    {
        immediate: true,
    },
);

function handleDialogClose() {
    if (props.open) emit("close");
}

async function handleSubmit() {
    loading.value = true;
    error.value = null;

    try {
        const tenant = await getClient().createTenant({
            comment: comment.value.trim() || null,
        });

        await tenantStore.loadTenants();
        tenantStore.syncCurrentTenant(tenant.id);

        toast.success("Tenant 已创建");
        emit("created", tenant.id);
    } catch (err) {
        error.value = err instanceof Error ? err.message : "创建 tenant 失败。";
        toast.error(error.value);
    } finally {
        loading.value = false;
    }
}
</script>

<template>
    <dialog ref="dialogRef" class="modal" @close="handleDialogClose">
        <div class="modal-box w-11/12 max-w-lg">
            <form method="dialog">
                <button
                    class="btn btn-circle btn-ghost btn-sm absolute right-2 top-2"
                    aria-label="Close"
                >
                    <CloseIcon class="h-4 w-4" />
                </button>
            </form>

            <h3 class="text-lg font-bold">创建 Tenant</h3>
            <p class="mt-1 text-sm text-base-content/70">
                输入 tenant 备注并创建新的 tenant。创建成功后会自动切换到新
                tenant 并进入 application 视图。
            </p>

            <form class="mt-4 space-y-4" @submit.prevent="handleSubmit">
                <label class="form-control w-full">
                    <textarea
                        v-model="comment"
                        class="textarea textarea-bordered min-h-28 w-full mb-8"
                        placeholder="可选，输入 tenant 备注"
                    ></textarea>
                </label>

                <div v-if="error" class="alert alert-error alert-soft">
                    <span>{{ error }}</span>
                </div>

                <button
                    type="submit"
                    class="btn btn-primary w-full"
                    :class="{ loading }"
                    :disabled="loading"
                >
                    创建 Tenant
                </button>
            </form>
        </div>

        <form method="dialog" class="modal-backdrop">
            <button aria-label="Close"></button>
        </form>
    </dialog>
</template>
