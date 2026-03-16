<script setup lang="ts">
import { computed, ref } from "vue";
import EntityListPage from "../components/EntityListPage.vue";

type ApplicationOption = {
    id: string;
    name: string;
    description: string;
};

const applicationOptions: ApplicationOption[] = [
    {
        id: "alpha-console",
        name: "Alpha Console",
        description: "内部管理端应用用户",
    },
    {
        id: "retail-portal",
        name: "Retail Portal",
        description: "面向商户的门户用户",
    },
    {
        id: "ops-workbench",
        name: "Ops Workbench",
        description: "运营工作台用户",
    },
];

const selectedApplicationId = ref("");

const selectedApplication = computed(
    () =>
        applicationOptions.find(
            (application) => application.id === selectedApplicationId.value,
        ) ?? null,
);

const summaryText = computed(() => {
    if (!selectedApplication.value) {
        return "请选择一个 application，用于查看该应用下的用户列表占位结构。";
    }

    return `当前查看 ${selectedApplication.value.name} 的用户管理占位内容，真实列表接口尚未接入。`;
});
</script>

<template>
    <EntityListPage
        page-title="Users"
        page-description="按 application 维度管理用户。当前页面先提供筛选、表格结构与空态说明，便于后续接入真实用户列表、详情和维护能力。"
        card-title="Application Users"
        :summary-text="summaryText"
    >
        <template #actions>
            <button type="button" class="btn btn-primary btn-sm" disabled>
                新增用户
            </button>
        </template>

        <div
            class="grid gap-6 px-6 pt-6 lg:grid-cols-[minmax(0,2fr)_minmax(18rem,1fr)]"
        >
            <label class="form-control w-full">
                <div class="label">
                    <span class="label-text">选择 Application</span>
                </div>
                <select
                    v-model="selectedApplicationId"
                    class="select select-bordered w-full"
                >
                    <option value="">请选择 application</option>
                    <option
                        v-for="application in applicationOptions"
                        :key="application.id"
                        :value="application.id"
                    >
                        {{ application.name }}
                    </option>
                </select>
            </label>

            <div class="rounded-box border border-base-200 bg-base-50/60 p-4">
                <div class="flex items-center justify-between gap-3">
                    <h3 class="text-sm font-medium text-base-content">
                        页面状态
                    </h3>
                    <span class="badge badge-warning badge-soft">待接入</span>
                </div>
                <p class="mt-2 text-sm text-base-content/70">
                    application 选择器当前使用本地占位选项。真实 application
                    列表和用户数据将在后续版本接入 SDK 与后端接口。
                </p>
                <p
                    v-if="selectedApplication"
                    class="mt-3 text-sm text-base-content/60"
                >
                    已选择
                    <span class="font-medium text-base-content">
                        {{ selectedApplication.name }}
                    </span>
                    ，{{ selectedApplication.description }}。
                </p>
            </div>
        </div>

        <div class="px-6 pb-6">
            <div
                v-if="!selectedApplication"
                class="alert alert-info alert-soft mt-6"
            >
                <span>
                    请先选择一个
                    application，然后在这里查看对应的用户列表、状态和后续操作入口。
                </span>
            </div>

            <div v-else class="mt-6 space-y-4">
                <div
                    class="rounded-box border border-base-200 bg-base-50/60 p-4"
                >
                    <div
                        class="flex flex-col gap-2 lg:flex-row lg:items-center lg:justify-between"
                    >
                        <div>
                            <h3 class="text-sm font-medium text-base-content">
                                {{ selectedApplication.name }} 用户列表
                            </h3>
                            <p class="mt-1 text-sm text-base-content/70">
                                当前仅提供列表结构预览。接入真实数据后，这里将展示用户标识、昵称、邮箱、手机号与状态。
                            </p>
                        </div>

                        <div class="badge badge-neutral badge-outline">
                            数据源未连接
                        </div>
                    </div>
                </div>

                <div class="overflow-x-auto rounded-box border border-base-200">
                    <table class="table min-w-220">
                        <thead>
                            <tr class="text-sm text-base-content/70">
                                <th class="whitespace-nowrap">User ID</th>
                                <th class="whitespace-nowrap">Nickname</th>
                                <th class="whitespace-nowrap">Email</th>
                                <th class="whitespace-nowrap">Phone</th>
                                <th class="whitespace-nowrap">Application</th>
                                <th class="whitespace-nowrap">Status</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                <td
                                    colspan="6"
                                    class="py-10 text-center text-sm text-base-content/60"
                                >
                                    {{ selectedApplication.name }}
                                    的用户列表尚未接入真实接口。当前页面已准备好筛选和表格结构，后续可直接接入该
                                    application 的用户查询能力。
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>

                <div class="alert alert-warning alert-soft">
                    <span>
                        当前不提供详情、编辑或删除入口，避免暴露尚未实现的无效操作。
                    </span>
                </div>
            </div>
        </div>
    </EntityListPage>
</template>
