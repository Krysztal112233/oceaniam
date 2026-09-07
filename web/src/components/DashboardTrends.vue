<script setup lang="ts">
import type { components } from "@oceaniam/sdk";
import {
  CategoryScale,
  Chart as ChartJS,
  Filler,
  Legend,
  LineElement,
  LinearScale,
  PointElement,
  Title,
  Tooltip,
  type ChartData,
  type ChartOptions,
} from "chart.js";
import { computed } from "vue";
import { Line } from "vue-chartjs";
import { useI18n } from "vue-i18n";

type Trends = components["schemas"]["PlatformTrendsVO"];

const props = defineProps<{ trends: Trends }>();
const { t, locale } = useI18n();

ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
  Filler,
);

const palette = ["#3b82f6", "#8b5cf6", "#10b981", "#f59e0b"];
const labels = computed(() =>
  props.trends.tenants.map((point) =>
    new Intl.DateTimeFormat(locale.value, {
      month: "short",
      day: "numeric",
    }).format(new Date(point.bucket)),
  ),
);
const data = computed<ChartData<"line">>(() => ({
  labels: labels.value,
  datasets: [
    {
      label: t("dashboard.tenants"),
      data: props.trends.tenants.map((p) => p.count),
      borderColor: palette[0],
      backgroundColor: `${palette[0]}22`,
      fill: true,
      tension: 0.35,
    },
    {
      label: t("dashboard.applications"),
      data: props.trends.applications.map((p) => p.count),
      borderColor: palette[1],
      backgroundColor: `${palette[1]}22`,
      tension: 0.35,
    },
    {
      label: t("dashboard.users"),
      data: props.trends.users.map((p) => p.count),
      borderColor: palette[2],
      backgroundColor: `${palette[2]}22`,
      tension: 0.35,
    },
    {
      label: t("dashboard.administrators"),
      data: props.trends.administrators.map((p) => p.count),
      borderColor: palette[3],
      backgroundColor: `${palette[3]}22`,
      tension: 0.35,
    },
  ],
}));
const options: ChartOptions<"line"> = {
  responsive: true,
  maintainAspectRatio: false,
  interaction: { intersect: false, mode: "index" },
  scales: { y: { beginAtZero: true, ticks: { precision: 0 } } },
  plugins: { legend: { position: "bottom" } },
};
</script>

<template>
  <div class="h-80">
    <Line :data="data" :options="options" />
  </div>
</template>
