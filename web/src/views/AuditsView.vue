<script setup lang="ts">
import { Icon } from "@iconify/vue/offline";
import { keepPreviousData, useQuery } from "@tanstack/vue-query";
import type { components } from "@oceaniam/sdk";
import { unwrap } from "@oceaniam/sdk";
import { ref } from "vue";
import { useI18n } from "vue-i18n";

import EmptyState from "@/components/EmptyState.vue";
import ErrorState from "@/components/ErrorState.vue";
import ModalDialog from "@/components/ModalDialog.vue";
import PageHeader from "@/components/PageHeader.vue";
import PaginationBar from "@/components/PaginationBar.vue";
import { api, AUTH_HEADER } from "@/services/api";

type Audit = components["schemas"]["AuditLogVO"];

const { locale } = useI18n();
const page = ref(1);
const auditType = ref("");
const selected = ref<Audit | null>(null);
const auditTypes = [
  "SignJwt",
  "RevokeJwt",
  "RefreshJwt",
  "CreateApplication",
  "PatchApplication",
  "PatchApplicationConfiguration",
  "DeleteApplication",
  "CreateTenants",
  "DeleteTenants",
  "PatchTenant",
  "CreateAdministrator",
  "PatchAdministrator",
  "CreateApplicationUser",
  "DeleteApplicationUser",
  "CreateApplicationSecret",
  "DeleteApplicationSecret",
  "BindApplicationSecret",
  "UnbindApplicationSecret",
  "CreateChallenge",
  "VerifyChallenge",
  "RotateKey",
  "RevokeKey",
  "CreateDevAccount",
  "DevAccountExpired",
];

const auditsQuery = useQuery({
  queryKey: ["audits", page, auditType],
  placeholderData: keepPreviousData,
  queryFn: async () => {
    const query: { page: number; per_page: number; audit_type?: string } = {
      page: page.value,
      per_page: 25,
    };
    if (auditType.value) query.audit_type = auditType.value;
    return unwrap(
      await api.GET("/audits", { params: { header: AUTH_HEADER, query } }),
    );
  },
});

function formatDate(value: string) {
  return new Intl.DateTimeFormat(locale.value, {
    dateStyle: "medium",
    timeStyle: "medium",
  }).format(new Date(value));
}

function payloadPreview(payload: unknown) {
  const text = JSON.stringify(payload);
  return text.length > 100 ? `${text.slice(0, 100)}…` : text;
}
</script>

<template>
  <div class="page-shell">
    <PageHeader :title="$t('audits.title')" :subtitle="$t('audits.subtitle')">
      <template #actions>
        <label class="select">
          <Icon
            icon="material-symbols:filter-list-rounded"
            class="size-5 opacity-50"
          />
          <select v-model="auditType" @change="page = 1">
            <option value="">{{ $t("audits.allTypes") }}</option>
            <option v-for="type in auditTypes" :key="type" :value="type">
              {{ type }}
            </option>
          </select>
        </label>
      </template>
    </PageHeader>

    <ErrorState
      v-if="auditsQuery.isError.value"
      :message="auditsQuery.error.value?.message"
      @retry="auditsQuery.refetch()"
    />
    <div v-else-if="auditsQuery.isPending.value" class="skeleton h-96 w-full" />
    <EmptyState
      v-else-if="!auditsQuery.data.value?.items.length"
      icon="material-symbols:manage-search-rounded"
      :title="$t('audits.empty')"
    />
    <div
      v-else
      class="overflow-x-auto rounded-box border border-base-300 bg-base-100 shadow-sm"
    >
      <table class="table table-zebra">
        <thead>
          <tr>
            <th>ID</th>
            <th>{{ $t("audits.type") }}</th>
            <th>{{ $t("audits.payload") }}</th>
            <th>{{ $t("audits.timestamp") }}</th>
            <th />
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="audit in auditsQuery.data.value.items"
            :key="audit.id"
            class="cursor-pointer hover"
            @click="selected = audit"
          >
            <td>
              <code class="text-xs">{{ audit.id }}</code>
            </td>
            <td>
              <span class="badge badge-outline whitespace-nowrap">{{
                audit.audit_type
              }}</span>
            </td>
            <td class="max-w-md truncate font-mono text-xs">
              {{ payloadPreview(audit.payload) }}
            </td>
            <td class="whitespace-nowrap text-sm">
              {{ formatDate(audit.created_at) }}
            </td>
            <td>
              <button
                class="btn btn-ghost btn-sm btn-square"
                :aria-label="$t('audits.inspect')"
              >
                <Icon icon="material-symbols:visibility-outline-rounded" />
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <PaginationBar
      v-if="auditsQuery.data.value"
      :page="page"
      :total="auditsQuery.data.value.page_info.total"
      :has-next="auditsQuery.data.value.page_info.has_next"
      @change="page = $event"
    />

    <ModalDialog
      :open="Boolean(selected)"
      :title="selected?.audit_type || $t('audits.payload')"
      wide
      @close="selected = null"
    >
      <div v-if="selected" class="space-y-4">
        <div class="flex flex-wrap gap-2">
          <span class="badge badge-outline">{{ selected.id }}</span
          ><span class="badge badge-ghost">{{
            formatDate(selected.created_at)
          }}</span>
        </div>
        <pre
          class="max-h-[65vh] overflow-auto rounded-box bg-neutral p-5 text-xs text-neutral-content"
          >{{ JSON.stringify(selected.payload, null, 2) }}</pre>
      </div>
    </ModalDialog>
  </div>
</template>
