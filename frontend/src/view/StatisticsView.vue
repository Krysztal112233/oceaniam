<script setup lang="ts">
import TenantIcon from "@iconify-vue/material-symbols/account-tree-outline-rounded";
import AppsIcon from "@iconify-vue/material-symbols/widgets-outline-rounded";
import AdminIcon from "@iconify-vue/material-symbols/admin-panel-settings-outline-rounded";
import PeopleIcon from "@iconify-vue/material-symbols/group-outline-rounded";
import KeyIcon from "@iconify-vue/material-symbols/key-outline-rounded";
import CalendarIcon from "@iconify-vue/material-symbols/calendar-month-outline-rounded";
import { computed, onMounted, ref } from "vue";
import type { OverviewVO, PlatformTrendsVO } from "@oceaniam/sdk";
import MetricsStats from "../components/metrics/MetricsStats.vue";
import MetricsStatsItem from "../components/metrics/MetricsStatsItem.vue";
import MetricsTrendCard from "../components/metrics/MetricsTrendCard.vue";
import { getClient } from "../utils/api-client";

type Granularity = "day" | "week" | "month";

const overview = ref<OverviewVO | null>(null);
const trends = ref<PlatformTrendsVO | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);
const requestId = ref(0);
const granularity = ref<Granularity>("day");
const rangeStart = ref("");
const rangeEnd = ref("");
const datePickerOpen = ref(false);

function todayStr(): string {
    const d = new Date();
    const month = String(d.getMonth() + 1).padStart(2, "0");
    const day = String(d.getDate()).padStart(2, "0");
    return `${d.getFullYear()}-${month}-${day}`;
}

function daysAgoStr(n: number): string {
    const d = new Date();
    d.setDate(d.getDate() - n);
    const month = String(d.getMonth() + 1).padStart(2, "0");
    const day = String(d.getDate()).padStart(2, "0");
    return `${d.getFullYear()}-${month}-${day}`;
}

rangeStart.value = daysAgoStr(30);
rangeEnd.value = todayStr();

function computedRange(): number {
    const start = new Date(rangeStart.value);
    const end = new Date(rangeEnd.value);
    const diff = end.getTime() - start.getTime();
    return Math.max(1, Math.ceil(diff / (1000 * 60 * 60 * 24)));
}

const numberFormatter = new Intl.NumberFormat("zh-CN");

const trendCards = computed(() => {
    if (!trends.value) return [];

    function buildCard(
        title: string,
        points: Array<{ bucket: string; count: bigint }>,
        lineColor: string,
        fillColor: string,
    ) {
        const padded = padTrendData(points, granularity.value, computedRange());
        return {
            title,
            labels: padded.labels,
            series: [
                {
                    label: title,
                    data: padded.values,
                    color: { line: lineColor, fill: fillColor },
                },
            ],
        };
    }

    return [
        buildCard("Tenants", trends.value.tenants, "#6366f1", "#6366f11a"),
        buildCard(
            "Applications",
            trends.value.applications,
            "#06b6d4",
            "#06b6d41a",
        ),
        buildCard("Users", trends.value.users, "#f59e0b", "#f59e0b1a"),
        buildCard(
            "Administrators",
            trends.value.administrators,
            "#ef4444",
            "#ef44441a",
        ),
    ];
});

function formatCount(value: bigint | number | undefined): string {
    if (value === undefined) return "—";
    return numberFormatter.format(Number(value));
}

function formatBucketDate(iso: string): string {
    try {
        const date = new Date(iso);
        const month = String(date.getMonth() + 1).padStart(2, "0");
        const day = String(date.getDate()).padStart(2, "0");
        return `${month}-${day}`;
    } catch {
        return iso;
    }
}

type TrendPoint = { bucket: string; count: bigint };

function padTrendData(
    points: TrendPoint[],
    granularity: Granularity,
    range: number,
): { labels: string[]; values: number[] } {
    const countByLabel = new Map<string, number>();
    for (const p of points) {
        countByLabel.set(formatBucketDate(p.bucket), Number(p.count));
    }

    const labels: string[] = [];
    const values: number[] = [];

    const now = new Date();
    now.setHours(0, 0, 0, 0);

    const start = new Date(now);
    start.setDate(start.getDate() - range);

    if (granularity === "week") {
        const day = start.getDay();
        const diff = day === 0 ? -6 : 1 - day;
        start.setDate(start.getDate() + diff);
    } else if (granularity === "month") {
        start.setDate(1);
    }

    const current = new Date(start);
    while (current <= now) {
        const label = formatBucketDate(current.toISOString());
        labels.push(label);
        values.push(countByLabel.get(label) ?? 0);

        if (granularity === "day") {
            current.setDate(current.getDate() + 1);
        } else if (granularity === "week") {
            current.setDate(current.getDate() + 7);
        } else {
            current.setMonth(current.getMonth() + 1);
        }
    }

    return { labels, values };
}

function formatGranularityLabel(g: Granularity): string {
    switch (g) {
        case "day":
            return "按日";
        case "week":
            return "按周";
        case "month":
            return "按月";
    }
}

function formatRangeLabel(): string {
    if (!rangeStart.value || !rangeEnd.value) return "";
    return `${rangeStart.value} ~ ${rangeEnd.value}`;
}

function applyDateRange(): void {
    datePickerOpen.value = false;
    void loadStatistics();
}

async function loadStatistics(): Promise<void> {
    const rid = requestId.value + 1;
    requestId.value = rid;
    loading.value = true;
    error.value = null;

    try {
        const statsResult = await getClient().getStatistics();

        if (rid !== requestId.value) return;

        overview.value = statsResult;
    } catch (err) {
        if (rid !== requestId.value) return;

        overview.value = null;
        trends.value = null;
        error.value = err instanceof Error ? err.message : "加载统计数据失败。";
        loading.value = false;
        return;
    }

    try {
        const trendsResult = await getClient().getStatisticsTrends({
            granularity: granularity.value,
            range: computedRange(),
        });

        if (rid !== requestId.value) return;

        trends.value = trendsResult;
    } catch {
        // trends are optional — show stats even if trends fail
    } finally {
        if (rid === requestId.value) {
            loading.value = false;
        }
    }
}

function onGranularityChange(g: Granularity): void {
    granularity.value = g;
    void loadStatistics();
}

onMounted(() => {
    void loadStatistics();
});
</script>

<template>
    <section class="flex flex-col gap-6">
        <header class="flex flex-col gap-2">
            <h1 class="text-2xl font-semibold text-base-content">Statistics</h1>
            <p class="max-w-2xl text-sm text-base-content/70">
                平台级统计指标和趋势数据，展示租户、应用、用户及管理员的增长趋势。
            </p>
        </header>

        <div v-if="loading && !overview" class="flex flex-col gap-6">
            <div class="card border border-base-200 bg-base-100 shadow-sm">
                <div class="card-body gap-4 p-6">
                    <div class="skeleton h-16 w-full"></div>
                </div>
            </div>
            <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
                <div class="skeleton h-72 w-full"></div>
                <div class="skeleton h-72 w-full"></div>
                <div class="skeleton h-72 w-full"></div>
                <div class="skeleton h-72 w-full"></div>
            </div>
        </div>

        <div
            v-else-if="error"
            class="card border border-base-200 bg-base-100 shadow-sm"
        >
            <div class="card-body">
                <div class="alert alert-error alert-soft">
                    <span>{{ error }}</span>
                </div>
            </div>
        </div>

        <template v-else>
            <div class="card border border-base-200 bg-base-100 shadow-sm">
                <div class="card-body gap-4 p-6">
                    <h2 class="text-lg font-medium text-base-content">
                        Overview
                    </h2>

                    <MetricsStats v-if="overview">
                        <MetricsStatsItem
                            title="Tenants"
                            :value="formatCount(overview.total_tenants)"
                            desc="平台租户总数"
                            :icon="TenantIcon"
                            figure-class="text-primary"
                        />
                        <MetricsStatsItem
                            title="Applications"
                            :value="formatCount(overview.total_applications)"
                            desc="平台应用总数"
                            :icon="AppsIcon"
                            figure-class="text-secondary"
                        />
                        <MetricsStatsItem
                            title="Users"
                            :value="
                                formatCount(overview.total_application_users)
                            "
                            desc="应用用户总数"
                            :icon="PeopleIcon"
                            figure-class="text-accent"
                        />
                        <MetricsStatsItem
                            title="Administrators"
                            :value="formatCount(overview.total_administrators)"
                            desc="平台管理员总数"
                            :icon="AdminIcon"
                            figure-class="text-warning"
                        />
                        <MetricsStatsItem
                            title="Active Secrets"
                            :value="formatCount(overview.total_active_secrets)"
                            desc="活跃密钥总数"
                            :icon="KeyIcon"
                            figure-class="text-info"
                        />
                    </MetricsStats>

                    <div v-else class="skeleton h-16 w-full"></div>
                </div>
            </div>

            <div
                class="flex flex-wrap items-center gap-4 rounded-box border border-base-200 bg-base-100 px-5 py-3 shadow-sm"
            >
                <div class="join">
                    <button
                        v-for="g in (['day', 'week', 'month'] as Granularity[])"
                        :key="g"
                        type="button"
                        class="btn btn-sm join-item"
                        :class="{ 'btn-active': granularity === g }"
                        :disabled="loading"
                        @click="onGranularityChange(g)"
                    >
                        {{ formatGranularityLabel(g) }}
                    </button>
                </div>

                <div class="relative flex items-center gap-2">
                    <button
                        type="button"
                        class="btn btn-ghost btn-sm gap-1"
                        :disabled="loading"
                        @click="datePickerOpen = !datePickerOpen"
                    >
                        <CalendarIcon width="16" height="16" />
                        <span class="text-sm">{{ formatRangeLabel() }}</span>
                    </button>

                    <div
                        v-if="datePickerOpen"
                        class="fixed inset-0 z-40"
                        @click="datePickerOpen = false"
                    ></div>

                    <div
                        v-if="datePickerOpen"
                        class="card card-sm absolute right-0 top-full z-50 mt-1 w-72 border border-base-200 bg-base-100 shadow-xl"
                    >
                        <div class="card-body gap-3">
                            <label class="form-control w-full">
                                <div class="label">
                                    <span
                                        class="label-text text-xs text-base-content/60"
                                    >
                                        开始日期
                                    </span>
                                </div>
                                <input
                                    v-model="rangeStart"
                                    type="date"
                                    class="input input-bordered input-sm w-full"
                                />
                            </label>

                            <label class="form-control w-full">
                                <div class="label">
                                    <span
                                        class="label-text text-xs text-base-content/60"
                                    >
                                        结束日期
                                    </span>
                                </div>
                                <input
                                    v-model="rangeEnd"
                                    type="date"
                                    class="input input-bordered input-sm w-full"
                                />
                            </label>

                            <button
                                type="button"
                                class="btn btn-primary btn-sm mt-1"
                                :disabled="loading"
                                @click="applyDateRange"
                            >
                                应用
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
                <div v-if="loading" class="skeleton h-72 w-full"></div>
                <MetricsTrendCard
                    v-for="card in trendCards"
                    :key="card.title"
                    :title="card.title"
                    :labels="card.labels"
                    :series="card.series"
                    :loading="loading"
                />
            </div>
        </template>
    </section>
</template>
