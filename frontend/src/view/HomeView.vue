<script setup lang="ts">
import AppsIcon from "@iconify-vue/material-symbols/apps";
import LoginIcon from "@iconify-vue/material-symbols/login";
import TokenIcon from "@iconify-vue/material-symbols/token-outline";
import { computed, ref } from "vue";
import MetricsStats from "../components/MetricsStats.vue";
import MetricsStatsItem from "../components/MetricsStatsItem.vue";
import MetricsTrendCard from "../components/MetricsTrendCard.vue";

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

type MetricsTimeWindow = "24h" | "7d" | "30d";

const timeWindow = ref<MetricsTimeWindow>("7d");

function formatMonthDay(date: Date): string {
    const month = String(date.getMonth() + 1).padStart(2, "0");
    const day = String(date.getDate()).padStart(2, "0");
    return `${month}-${day}`;
}

function formatHour(date: Date): string {
    return String(date.getHours()).padStart(2, "0");
}

// NOTE: THIS IS MOCKING DATA GENERATOR.
function lastNDaysLabels(days: number): string[] {
    const now = new Date();
    return Array.from({ length: days }, (_, index) => {
        const date = new Date(now);
        date.setDate(now.getDate() - (days - 1 - index));
        return formatMonthDay(date);
    });
}

// NOTE: THIS IS MOCKING DATA GENERATOR.
function lastNHoursLabels(hours: number): string[] {
    const now = new Date();
    return Array.from({ length: hours }, (_, index) => {
        const date = new Date(now);
        date.setHours(now.getHours() - (hours - 1 - index));
        return formatHour(date);
    });
}

// NOTE: THIS IS MOCKING DATA GENERATOR.
function generateSeries(
    length: number,
    base: number,
    variance: number,
): number[] {
    return Array.from({ length }, (_, index) => {
        const wave = Math.sin(index * 0.9) * 0.6 + Math.cos(index * 0.35) * 0.4;
        const value = base + variance * (0.5 + 0.5 * wave) + (index % 3) * 3;
        return Math.max(0, Math.round(value));
    });
}

const trendLabels = computed(() => {
    switch (timeWindow.value) {
        case "24h":
            return lastNHoursLabels(24);
        case "30d":
            return lastNDaysLabels(30);
        case "7d":
        default:
            return lastNDaysLabels(7);
    }
});

const trendSeries = computed(() => {
    const length = trendLabels.value.length;

    const loginBase = timeWindow.value === "24h" ? 18 : 140;
    const loginVariance = timeWindow.value === "24h" ? 35 : 90;
    const loginSeries = generateSeries(length, loginBase, loginVariance);

    const jwtSeries = loginSeries.map((value, index) =>
        Math.round(value * (1.9 + (index % 4) * 0.08)),
    );

    // NOTE: CHANGE THIS IN FUTURE
    return [
        {
            label: "登录次数",
            data: loginSeries,
            // color: {
            //     line: "hsl(var(--p))",
            //     fill: "hsl(var(--p) / 0.20)",
            // },

            color: {
                line: "green",
                fill: "green",
            },
        },
        {
            label: "JWT 颁发数量",
            data: jwtSeries,
            // color: {
            //     line: "hsl(var(--a))",
            //     fill: "hsl(var(--a) / 0.20)",
            // },
            color: {
                line: "lightgreen",
                fill: "lightgreen",
            },
        },
    ];
});
</script>

<template>
    <div class="flex flex-col gap-4">
        <MetricsStats>
            <MetricsStatsItem
                title="应用数量"
                :value="formatNumber(metrics.applicationsCount)"
                desc="当前系统内应用总数"
                :icon="AppsIcon"
                figure-class="text-primary"
            />
            <MetricsStatsItem
                title="过去 24 小时总登录次数"
                :value="formatNumber(metrics.totalLoginsLast24h)"
                desc="统计窗口：过去 24 小时"
                :icon="LoginIcon"
                figure-class="text-primary"
            />
            <MetricsStatsItem
                title="过去 24 小时 JWT 总颁发数量"
                :value="formatNumber(metrics.totalJwtIssuedLast24h)"
                desc="统计窗口：过去 24 小时"
                :icon="TokenIcon"
                figure-class="text-primary"
            />
        </MetricsStats>

        <MetricsTrendCard
            title="趋势（登录次数 / JWT 颁发数量）"
            :labels="trendLabels"
            :series="trendSeries"
        >
            <template #actions>
                <div class="tabs tabs-boxed tabs-sm">
                    <button
                        type="button"
                        class="tab"
                        :class="{ 'tab-active': timeWindow === '24h' }"
                        @click="timeWindow = '24h'"
                    >
                        24h
                    </button>
                    <button
                        type="button"
                        class="tab"
                        :class="{ 'tab-active': timeWindow === '7d' }"
                        @click="timeWindow = '7d'"
                    >
                        7d
                    </button>
                    <button
                        type="button"
                        class="tab"
                        :class="{ 'tab-active': timeWindow === '30d' }"
                        @click="timeWindow = '30d'"
                    >
                        30d
                    </button>
                </div>
            </template>
        </MetricsTrendCard>
    </div>
</template>
