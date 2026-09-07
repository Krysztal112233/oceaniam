<script setup lang="ts">
import { Icon } from "@iconify/vue/offline";
import { useQuery } from "@tanstack/vue-query";
import { unwrap } from "@oceaniam/sdk";
import { computed } from "vue";

import ErrorState from "@/components/ErrorState.vue";
import { api, AUTH_HEADER } from "@/services/api";

const props = defineProps<{ tenantId: string; applicationId: string }>();

const statisticsQuery = useQuery({
  queryKey: computed(() => [
    "tenant",
    props.tenantId,
    "application",
    props.applicationId,
    "statistics",
  ]),
  queryFn: async () =>
    unwrap(
      await api.GET(
        "/tenants/{tenant_id}/applications/{application_id}/statistics",
        {
          params: {
            header: AUTH_HEADER,
            path: {
              tenant_id: props.tenantId,
              application_id: props.applicationId,
            },
          },
        },
      ),
    ),
});
</script>

<template>
  <ErrorState
    v-if="statisticsQuery.isError.value"
    :message="statisticsQuery.error.value?.message"
    @retry="statisticsQuery.refetch()"
  />
  <div v-else class="grid gap-4 md:grid-cols-3">
    <div class="stat rounded-box bg-base-200">
      <div class="stat-figure text-primary">
        <Icon icon="material-symbols:group-outline-rounded" class="size-7" />
      </div>
      <div class="stat-title">
        {{ $t("applications.usersTotal") }}
      </div>
      <div
        v-if="statisticsQuery.isPending.value"
        class="skeleton mt-2 h-8 w-20"
      />
      <div v-else class="stat-value text-3xl">
        {{ statisticsQuery.data.value?.total_users.toLocaleString() }}
      </div>
    </div>
    <div class="stat rounded-box bg-base-200">
      <div class="stat-figure text-success">
        <Icon icon="material-symbols:key-outline-rounded" class="size-7" />
      </div>
      <div class="stat-title">
        {{ $t("dashboard.secrets") }}
      </div>
      <div
        v-if="statisticsQuery.isPending.value"
        class="skeleton mt-2 h-8 w-20"
      />
      <div v-else class="stat-value text-3xl">
        {{ statisticsQuery.data.value?.total_active_keys.toLocaleString() }}
      </div>
    </div>
    <div class="rounded-box bg-base-200 p-5">
      <div class="text-base-content/55 text-sm">
        {{ $t("applications.id") }}
      </div>
      <code class="mt-2 block break-all font-semibold">{{
        applicationId
      }}</code>
    </div>
  </div>
</template>
