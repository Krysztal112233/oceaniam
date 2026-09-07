<script setup lang="ts">
import { Icon } from "@iconify/vue/offline";
import { useQuery } from "@tanstack/vue-query";

import DashboardTrends from "@/components/DashboardTrends.vue";
import ErrorState from "@/components/ErrorState.vue";
import PageHeader from "@/components/PageHeader.vue";
import { api, AUTH_HEADER } from "@/services/api";
import { unwrap } from "@oceaniam/sdk";

const overviewQuery = useQuery({
  queryKey: ["dashboard", "overview"],
  queryFn: async () =>
    unwrap(await api.GET("/statistics", { params: { header: AUTH_HEADER } })),
});
const trendsQuery = useQuery({
  queryKey: ["dashboard", "trends", 30],
  queryFn: async () =>
    unwrap(
      await api.GET("/statistics/trends", {
        params: {
          header: AUTH_HEADER,
          query: { granularity: "day", range: 30 },
        },
      }),
    ),
});

const stats = [
  {
    key: "total_tenants",
    label: "dashboard.tenants",
    icon: "material-symbols:domain-rounded",
    color: "text-primary",
  },
  {
    key: "total_applications",
    label: "dashboard.applications",
    icon: "material-symbols:apps-rounded",
    color: "text-secondary",
  },
  {
    key: "total_administrators",
    label: "dashboard.administrators",
    icon: "material-symbols:admin-panel-settings-outline-rounded",
    color: "text-accent",
  },
  {
    key: "total_application_users",
    label: "dashboard.users",
    icon: "material-symbols:group-outline-rounded",
    color: "text-success",
  },
  {
    key: "total_active_secrets",
    label: "dashboard.secrets",
    icon: "material-symbols:key-outline-rounded",
    color: "text-warning",
  },
] as const;
</script>

<template>
  <div class="page-shell">
    <PageHeader
      :title="$t('dashboard.title')"
      :subtitle="$t('dashboard.subtitle')"
    />

    <ErrorState
      v-if="overviewQuery.isError.value"
      :message="overviewQuery.error.value?.message"
      @retry="overviewQuery.refetch()"
    />
    <div v-else class="grid gap-4 sm:grid-cols-2 xl:grid-cols-5">
      <article
        v-for="stat in stats"
        :key="stat.key"
        class="stat rounded-box border border-base-300 bg-base-100 shadow-sm"
      >
        <div class="stat-figure" :class="stat.color">
          <Icon :icon="stat.icon" class="size-8" />
        </div>
        <div class="stat-title">
          {{ $t(stat.label) }}
        </div>
        <div
          v-if="overviewQuery.isPending.value"
          class="skeleton mt-2 h-9 w-20"
        />
        <div v-else class="stat-value text-3xl">
          {{ overviewQuery.data.value?.[stat.key].toLocaleString() }}
        </div>
      </article>
    </div>

    <section class="card mt-6 border border-base-300 bg-base-100 shadow-sm">
      <div class="card-body">
        <div class="flex items-center justify-between gap-3">
          <h2 class="card-title">
            <Icon icon="material-symbols:monitoring-rounded" />
            {{ $t("dashboard.trends") }}
          </h2>
          <span class="badge badge-ghost">{{
            $t("dashboard.last30Days")
          }}</span>
        </div>
        <ErrorState
          v-if="trendsQuery.isError.value"
          class="mt-4"
          :message="trendsQuery.error.value?.message"
          @retry="trendsQuery.refetch()"
        />
        <div
          v-else-if="trendsQuery.isPending.value"
          class="skeleton mt-4 h-80 w-full"
        />
        <DashboardTrends
          v-else-if="trendsQuery.data.value"
          class="mt-4"
          :trends="trendsQuery.data.value"
        />
      </div>
    </section>
  </div>
</template>
