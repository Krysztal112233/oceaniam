<script setup lang="ts">
import { computed } from "vue";
import { Line } from "vue-chartjs";
import type { ChartData, ChartOptions, TooltipItem } from "chart.js";
import {
    CategoryScale,
    Chart as ChartJS,
    Filler,
    Legend,
    LinearScale,
    LineElement,
    PointElement,
    Tooltip,
} from "chart.js";

ChartJS.register(
    CategoryScale,
    LinearScale,
    PointElement,
    LineElement,
    Filler,
    Tooltip,
    Legend,
);

type MetricsTrendSeries = {
    label: string;
    data: number[];
    color: {
        line: string;
        fill: string;
    };
};

const props = withDefaults(
    defineProps<{
        title: string;
        labels: string[];
        series: MetricsTrendSeries[];
        loading?: boolean;
    }>(),
    {
        loading: false,
    },
);

const numberFormatter = new Intl.NumberFormat("zh-CN");

function resolveDaisyHslColor(input: string): string {
    if (typeof document === "undefined") return input;

    const match = input.match(
        /^hsl\(\s*var\(\s*(--[\w-]+)\s*\)\s*(?:\/\s*([\d.]+)\s*)?\)$/i,
    );
    if (!match) return input;

    const variableName = match[1];
    if (!variableName) return input;
    const alpha = match[2] ? Number(match[2]) : null;

    const rawHsl = getComputedStyle(document.documentElement)
        .getPropertyValue(variableName)
        .trim();
    if (!rawHsl) return input;

    const [h, s, l] = rawHsl.split(/\s+/);
    if (!h || !s || !l) return input;

    if (alpha === null || Number.isNaN(alpha)) {
        return `hsl(${h}, ${s}, ${l})`;
    }

    return `hsla(${h}, ${s}, ${l}, ${alpha})`;
}

const chartData = computed<ChartData<"line">>(() => ({
    labels: props.labels,
    datasets: props.series.map((serie) => ({
        label: serie.label,
        data: serie.data,
        borderColor: resolveDaisyHslColor(serie.color.line),
        backgroundColor: resolveDaisyHslColor(serie.color.fill),
        fill: true,
        tension: 0.35,
        pointRadius: 2,
        pointHoverRadius: 4,
        borderWidth: 2,
    })),
}));

const chartOptions = computed<ChartOptions<"line">>(() => ({
    responsive: true,
    maintainAspectRatio: false,
    color: resolveDaisyHslColor("hsl(var(--bc))"),
    interaction: {
        mode: "index",
        intersect: false,
    },
    plugins: {
        legend: {
            display: false,
        },
        tooltip: {
            mode: "index",
            intersect: false,
            callbacks: {
                label: (context: TooltipItem<"line">) => {
                    const label = context.dataset.label ?? "";
                    const value = context.parsed.y;
                    if (value === null) return label;
                    return `${label}: ${numberFormatter.format(value)}`;
                },
            },
        },
    },
    scales: {
        x: {
            grid: {
                display: false,
            },
            ticks: {
                maxRotation: 0,
            },
        },
        y: {
            beginAtZero: true,
        },
    },
}));
</script>

<template>
    <!-- Card -->
    <div class="card bg-base-100 shadow">
        <!-- Card Body -->
        <div class="card-body gap-4">
            <div class="flex items-start justify-between gap-4">
                <div class="min-w-0">
                    <!-- Card Title -->
                    <h2 class="card-title">{{ title }}</h2>
                    <div
                        v-if="series.length"
                        class="mt-1 flex flex-wrap items-center gap-x-4 gap-y-1 text-xs opacity-80"
                    >
                        <div
                            v-for="serie in series"
                            :key="serie.label"
                            class="inline-flex items-center gap-2"
                        >
                            <span
                                class="h-2 w-2 rounded-full"
                                :style="{ backgroundColor: serie.color.line }"
                            />
                            <span class="truncate">{{ serie.label }}</span>
                        </div>
                    </div>
                </div>

                <div class="shrink-0">
                    <slot name="actions"></slot>
                </div>
            </div>

            <!-- If data still loading, display skeleton instead -->
            <div v-if="loading" class="skeleton h-64 w-full"></div>

            <div v-else class="h-64 w-full">
                <Line
                    :data="chartData"
                    :options="chartOptions"
                    dataset-id-key="label"
                />
            </div>
        </div>
    </div>
</template>
