<script setup lang="ts">
import AppsIcon from "@iconify-vue/material-symbols/apps";
import LoginIcon from "@iconify-vue/material-symbols/login";
import TokenIcon from "@iconify-vue/material-symbols/token-outline";
import { ref } from "vue";
import MetricsStats from "../components/MetricsStats.vue";
import MetricsStatsItem from "../components/MetricsStatsItem.vue";

type DashboardMetrics = {
    applicationsCount: number;
    totalLoginsLast24h: number;
    totalJwtIssuedLast24h: number;
};

const metrics = ref<DashboardMetrics>({
    applicationsCount: 0,
    totalLoginsLast24h: 0,
    totalJwtIssuedLast24h: 0,
});

const numberFormatter = new Intl.NumberFormat("zh-CN");

function formatNumber(value: number): string {
    return numberFormatter.format(value);
}
</script>

<template>
    <div class="flex flex-col gap-4">
        <MetricsStats>
            <MetricsStatsItem
                title="应用数量"
                :value="formatNumber(metrics.applicationsCount)"
                desc="当前系统内应用总数"
                :icon="AppsIcon"
                figure-class="text-secondary"
            />
            <MetricsStatsItem
                title="过去 24 小时总登录次数"
                :value="formatNumber(metrics.totalLoginsLast24h)"
                desc="统计窗口：过去 24 小时"
                :icon="LoginIcon"
                figure-class="text-secondary"
            />
            <MetricsStatsItem
                title="过去 24 小时 JWT 总颁发数量"
                :value="formatNumber(metrics.totalJwtIssuedLast24h)"
                desc="统计窗口：过去 24 小时"
                :icon="TokenIcon"
                figure-class="text-accent"
            />
        </MetricsStats>
    </div>
</template>
