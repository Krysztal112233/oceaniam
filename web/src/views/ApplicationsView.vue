<script setup lang="ts">
import { Icon } from "@iconify/vue/offline";
import {
  keepPreviousData,
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/vue-query";
import type { components } from "@oceaniam/sdk";
import { unwrap } from "@oceaniam/sdk";
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import ApplicationCard from "@/components/applications/ApplicationCard.vue";
import EmptyState from "@/components/EmptyState.vue";
import ErrorState from "@/components/ErrorState.vue";
import ModalDialog from "@/components/ModalDialog.vue";
import PageHeader from "@/components/PageHeader.vue";
import PaginationBar from "@/components/PaginationBar.vue";
import { api, AUTH_HEADER } from "@/services/api";
import { useTenantStore } from "@/stores/tenant";
import { useToastsStore } from "@/stores/toasts";

type Application = components["schemas"]["ApplicationVO"];

const tenant = useTenantStore();
const queryClient = useQueryClient();
const toasts = useToastsStore();
const { t } = useI18n();
const page = ref(1);
const expandedId = ref<string | null>(null);
const createOpen = ref(false);
const comment = ref("");
const tenantId = computed(() => tenant.currentId);

watch(
  () => tenant.revision,
  () => {
    page.value = 1;
    expandedId.value = null;
  },
);

const applicationsQuery = useQuery({
  queryKey: computed(() => [
    "tenant",
    tenantId.value,
    "applications",
    page.value,
  ]),
  enabled: computed(() => Boolean(tenantId.value)),
  placeholderData: keepPreviousData,
  queryFn: async () => {
    if (!tenantId.value) throw new Error("No tenant selected");
    return unwrap(
      await api.GET("/tenants/{tenant_id}/applications", {
        params: {
          header: AUTH_HEADER,
          path: { tenant_id: tenantId.value },
          query: { page: page.value, per_page: 20 },
        },
      }),
    );
  },
});

const createMutation = useMutation({
  mutationFn: async () => {
    if (!tenantId.value) throw new Error("No tenant selected");
    return unwrap(
      await api.POST("/tenants/{tenant_id}/applications", {
        params: { header: AUTH_HEADER, path: { tenant_id: tenantId.value } },
        body: { comment: comment.value.trim() || null },
      }),
    );
  },
  onSuccess: async (created) => {
    createOpen.value = false;
    comment.value = "";
    expandedId.value = created.application_id;
    await queryClient.invalidateQueries({
      queryKey: ["tenant", tenantId.value, "applications"],
    });
    toasts.show(t("applications.create"), "success");
  },
  onError: (error) =>
    toasts.show(
      error instanceof Error ? error.message : t("common.requestFailed"),
      "error",
    ),
});

async function applicationChanged() {
  await queryClient.invalidateQueries({
    queryKey: ["tenant", tenantId.value, "applications"],
  });
}

function toggle(app: Application) {
  expandedId.value = expandedId.value === app.id ? null : app.id;
}
</script>

<template>
  <div class="page-shell">
    <PageHeader
      :title="$t('applications.title')"
      :subtitle="$t('applications.subtitle')"
    >
      <template #actions>
        <button
          class="btn btn-primary"
          :disabled="!tenantId"
          @click="createOpen = true"
        >
          <Icon icon="material-symbols:add-rounded" class="size-5" />
          {{ $t("applications.create") }}
        </button>
      </template>
    </PageHeader>

    <EmptyState
      v-if="!tenantId"
      icon="material-symbols:domain-disabled-outline-rounded"
      :title="$t('tenant.none')"
      :description="$t('tenant.empty')"
    />
    <ErrorState
      v-else-if="applicationsQuery.isError.value"
      :message="applicationsQuery.error.value?.message"
      @retry="applicationsQuery.refetch()"
    />
    <div v-else-if="applicationsQuery.isPending.value" class="space-y-3">
      <div v-for="index in 4" :key="index" class="skeleton h-24 w-full" />
    </div>
    <EmptyState
      v-else-if="!applicationsQuery.data.value?.items.length"
      icon="material-symbols:apps-rounded"
      :title="$t('applications.empty')"
    >
      <button class="btn btn-primary btn-sm" @click="createOpen = true">
        {{ $t("applications.create") }}
      </button>
    </EmptyState>
    <div v-else class="space-y-3">
      <ApplicationCard
        v-for="application in applicationsQuery.data.value.items"
        :key="application.id"
        :application="application"
        :expanded="expandedId === application.id"
        @toggle="toggle(application)"
        @changed="applicationChanged"
        @deleted="
          expandedId = null;
          applicationChanged();
        "
      />
      <PaginationBar
        :page="page"
        :total="applicationsQuery.data.value.page_info.total"
        :has-next="applicationsQuery.data.value.page_info.has_next"
        @change="
          page = $event;
          expandedId = null;
        "
      />
    </div>

    <ModalDialog
      :open="createOpen"
      :title="$t('applications.create')"
      @close="createOpen = false"
    >
      <form @submit.prevent="createMutation.mutate()">
        <label class="fieldset">
          <span class="fieldset-legend">{{ $t("applications.comment") }}</span>
          <textarea v-model="comment" class="textarea w-full" rows="3" />
        </label>
        <div class="modal-action">
          <button
            type="button"
            class="btn"
            :disabled="createMutation.isPending.value"
            @click="createOpen = false"
          >
            {{ $t("common.cancel") }}
          </button>
          <button
            class="btn btn-primary"
            :disabled="createMutation.isPending.value"
          >
            <span
              v-if="createMutation.isPending.value"
              class="loading loading-spinner loading-sm"
            />
            {{ $t("common.create") }}
          </button>
        </div>
      </form>
    </ModalDialog>
  </div>
</template>
