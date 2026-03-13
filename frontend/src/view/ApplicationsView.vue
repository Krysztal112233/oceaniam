<script setup lang="ts">
import ApplicationTable from "../components/ApplicationTable.vue";
import EntityListPage from "../components/EntityListPage.vue";
import type { ApplicationVO } from "../../packages/sdk/src/types/ApplicationVO";

const applications: ApplicationVO[] = [
    {
        id: "app_01nk8h2p9w6q4z7r5t1y3u8v0x",
        comment: "主站登录与 OAuth 授权应用",
        tenant_id: "tenant_default",
    },
    {
        id: "app_01nk8h4dr9m2x6c1v7b5n3q8kp",
        comment: "内部管理后台",
        tenant_id: "tenant_default",
    },
    {
        id: "app_01nk8h6yt3f9s4j2l8p1d5m7ra",
        comment: null,
        tenant_id: "tenant_default",
    },
];

function handleDetail(applicationId: string) {
    console.debug("application detail clicked:", applicationId);
}

function handleDelete(applicationId: string) {
    console.debug("application delete clicked:", applicationId);
}
</script>

<template>
    <EntityListPage
        page-title="Applications"
        page-description="管理当前租户下的 application 列表，展示应用标识、备注信息，以及常用操作入口。"
        card-title="Application List"
        :summary-text="`共 ${applications.length} 个 application`"
    >
        <ApplicationTable
            :applications="applications"
            @detail="handleDetail"
            @delete="handleDelete"
        />
    </EntityListPage>
</template>
