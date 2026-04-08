<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { useAuthStore } from "../stores/auth";
import { useTenantStore } from "../stores/tenant";
import { getClient } from "../utils/api-client";

const router = useRouter();
const authStore = useAuthStore();
const tenantStore = useTenantStore();
const { hasTenants, tenantsError, tenantsLoading } = storeToRefs(tenantStore);
const resolveError = ref<string | null>(null);

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

    if (resolveError.value) {
        return resolveError.value;
    }

    if (!hasTenants.value) {
        return "当前没有可用 tenant，无法进入 User Management 页面。";
    }

    return "正在进入 User Management 页面...";
});

async function resolveUsersEntry() {
    resolveError.value = null;

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

    const response = await getClient().getApplications({
        tenant_id: tenantId,
        page: 1n,
        per_page: 1n,
    });

    const applicationId = response.items[0]?.id ?? "";
    if (!applicationId) {
        resolveError.value =
            "当前 tenant 下没有可用 application，无法进入 User Management 页面。";
        return;
    }

    await router.replace({
        name: "application-users",
        params: { tenantId, applicationId },
    });
}

onMounted(() => {
    void resolveUsersEntry();
});
</script>

<template>
    <section class="flex flex-col gap-6">
        <header class="flex flex-col gap-2">
            <h1 class="text-2xl font-semibold text-base-content">
                User Management
            </h1>
            <p class="max-w-2xl text-sm text-base-content/70">
                从最近使用的 tenant 恢复 User Management
                视图，并定位到一个可用的 application。
            </p>
        </header>

        <div class="card border border-base-200 bg-base-100 shadow-sm">
            <div class="card-body">
                <div
                    class="alert"
                    :class="
                        tenantsError || resolveError
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
