<script setup lang="ts">
import AddBoxIcon from "@iconify-vue/material-symbols/add-box-outline-rounded";
import AnalyticsIcon from "@iconify-vue/material-symbols/analytics-outline-rounded";
import HomeActionCard from "../components/HomeActionCard.vue";
</script>

<template>
    <section class="flex flex-col gap-6">
        <header class="flex flex-col gap-2">
            <h1 class="text-2xl font-semibold text-base-content">Home</h1>
            <p class="max-w-2xl text-sm text-base-content/70">
                从这里开始常用操作。你可以进入应用管理流程，或者查看当前系统报告。
            </p>
        </header>

        <div class="grid gap-4 lg:grid-cols-2">
            <HomeActionCard
                to="/applications"
                title="创建新的应用"
                description="进入 application 视图并基于当前 tenant 开始创建新的应用。"
                cta-label="前往 Applications"
                :icon="AddBoxIcon"
                accent="primary"
            />

            <HomeActionCard
                to="/statistics"
                title="查看系统报告"
                description="查看统计分析与趋势页面，快速了解系统当前的整体运行情况。"
                cta-label="查看 Statistics"
                :icon="AnalyticsIcon"
                accent="secondary"
            />
        </div>
    </section>
</template>
