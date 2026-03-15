<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, onMounted } from "vue";
import { useRouter } from "vue-router";
import { useAuthStore } from "../stores/auth";
import { useTenantStore } from "../stores/tenant";

const router = useRouter();
const authStore = useAuthStore();
const tenantStore = useTenantStore();
const { hasTenants, tenantsError, tenantsLoading } = storeToRefs(tenantStore);

const message = computed(() => {
    if (!authStore.isLoggedIn) {
        return "请先登录后再选择 tenant。";
    }

    if (tenantsLoading.value) {
        return "正在恢复上次使用的 tenant...";
    }

    if (tenantsError.value) {
        return tenantsError.value;
    }

    if (!hasTenants.value) {
        return "当前没有可用 tenant，无法进入 application 列表。";
    }

    return "正在进入 application 列表...";
});

async function resolveApplicationsEntry() {
    if (!authStore.isLoggedIn) {
        return;
    }

    if (!tenantStore.hasTenants) {
        await tenantStore.loadTenants();
    }

    const tenantId = tenantStore.ensureCurrentTenant();
    if (!tenantId) {
        return;
    }

    await router.replace({
        name: "applications",
        params: { tenantId },
    });
}

onMounted(() => {
    void resolveApplicationsEntry();
});
</script>

<template>
    <section class="flex flex-col gap-6">
        <header class="flex flex-col gap-2">
            <h1 class="text-2xl font-semibold text-base-content">
                Applications
            </h1>
            <p class="max-w-2xl text-sm text-base-content/70">
                从最近使用的 tenant 恢复 application 视图。
            </p>
        </header>

        <div class="card border border-base-200 bg-base-100 shadow-sm">
            <div class="card-body">
                <div
                    class="alert"
                    :class="
                        tenantsError
                            ? 'alert-error alert-soft'
                            : 'alert-info alert-soft'
                    "
                >
                    <span>{{ message }}</span>
                </div>
            </div>
        </div>
    </section>
</template>
