<script setup lang="ts">
import { Icon } from "@iconify/vue/offline";
import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import type { components } from "@oceaniam/sdk";
import { unwrap } from "@oceaniam/sdk";
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

import ConfirmDialog from "@/components/ConfirmDialog.vue";
import EmptyState from "@/components/EmptyState.vue";
import ErrorState from "@/components/ErrorState.vue";
import ModalDialog from "@/components/ModalDialog.vue";
import PageHeader from "@/components/PageHeader.vue";
import { api, AUTH_HEADER } from "@/services/api";
import { useTenantStore } from "@/stores/tenant";
import { useToastsStore } from "@/stores/toasts";

type SigningKey = components["schemas"]["ApplicationKeyVO"];

const tenant = useTenantStore();
const toasts = useToastsStore();
const queryClient = useQueryClient();
const { t, locale } = useI18n();
const revokeKey = ref<SigningKey | null>(null);
const jwksOpen = ref(false);
const tenantId = computed(() => tenant.currentId);
const queryKey = computed(() => ["tenant", tenantId.value, "signing-keys"]);

const keysQuery = useQuery({
  queryKey,
  enabled: computed(() => Boolean(tenantId.value)),
  queryFn: async () => {
    if (!tenantId.value) throw new Error("No tenant selected");
    return unwrap(
      await api.GET("/tenants/{tenant_id}/keys", {
        params: { header: AUTH_HEADER, path: { tenant_id: tenantId.value } },
      }),
    );
  },
});

const jwksQuery = useQuery({
  queryKey: computed(() => ["tenant", tenantId.value, "jwks"]),
  enabled: computed(() => jwksOpen.value && Boolean(tenantId.value)),
  queryFn: async () => {
    if (!tenantId.value) throw new Error("No tenant selected");
    return unwrap(
      await api.GET("/tenants/{tenant_id}/.well-known/jwks.json", {
        params: { path: { tenant_id: tenantId.value } },
      }),
    );
  },
});

const rotateMutation = useMutation({
  mutationFn: async () => {
    if (!tenantId.value) throw new Error("No tenant selected");
    return unwrap(
      await api.POST("/tenants/{tenant_id}/keys", {
        params: { header: AUTH_HEADER, path: { tenant_id: tenantId.value } },
      }),
    );
  },
  onSuccess: async () => {
    await queryClient.invalidateQueries({ queryKey: queryKey.value });
    await queryClient.invalidateQueries({
      queryKey: ["tenant", tenantId.value, "jwks"],
    });
    toasts.show(t("keys.rotate"), "success");
  },
  onError: (error) =>
    toasts.show(
      error instanceof Error ? error.message : t("common.requestFailed"),
      "error",
    ),
});

const revokeMutation = useMutation({
  mutationFn: async () => {
    if (!tenantId.value || !revokeKey.value) throw new Error("No key selected");
    return unwrap(
      await api.DELETE("/tenants/{tenant_id}/keys/{key_id}", {
        params: {
          header: AUTH_HEADER,
          path: { tenant_id: tenantId.value, key_id: revokeKey.value.key_id },
        },
      }),
    );
  },
  onSuccess: async () => {
    revokeKey.value = null;
    await queryClient.invalidateQueries({ queryKey: queryKey.value });
    await queryClient.invalidateQueries({
      queryKey: ["tenant", tenantId.value, "jwks"],
    });
    toasts.show(t("keys.revoke"), "success");
  },
  onError: (error) =>
    toasts.show(
      error instanceof Error ? error.message : t("common.requestFailed"),
      "error",
    ),
});

function formatDate(value: string) {
  return new Intl.DateTimeFormat(locale.value, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}

function formatStatus(value: string) {
  const messages: Record<string, string> = {
    active: "keys.active",
    pending: "keys.pending",
    retired: "keys.retired",
    revoked: "keys.revoked",
  };
  const message = messages[value.toLowerCase()];
  return message ? t(message) : value;
}

async function copyJwks() {
  if (!jwksQuery.data.value) return;
  await navigator.clipboard.writeText(
    JSON.stringify(jwksQuery.data.value, null, 2),
  );
  toasts.show(t("common.copied"), "success");
}
</script>

<template>
  <div class="page-shell">
    <PageHeader :title="$t('keys.title')" :subtitle="$t('keys.subtitle')">
      <template #actions>
        <button class="btn" :disabled="!tenantId" @click="jwksOpen = true">
          <Icon icon="material-symbols:data-object-rounded" />
          {{ $t("keys.jwks") }}
        </button>
        <button
          class="btn btn-primary"
          :disabled="!tenantId || rotateMutation.isPending.value"
          @click="rotateMutation.mutate()"
        >
          <span
            v-if="rotateMutation.isPending.value"
            class="loading loading-spinner loading-sm"
          /><Icon v-else icon="material-symbols:autorenew-rounded" />
          {{ $t("keys.rotate") }}
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
      v-else-if="keysQuery.isError.value"
      :message="keysQuery.error.value?.message"
      @retry="keysQuery.refetch()"
    />
    <div v-else-if="keysQuery.isPending.value" class="skeleton h-64 w-full" />
    <EmptyState
      v-else-if="!keysQuery.data.value?.items.length"
      icon="material-symbols:signature-rounded"
      :title="$t('keys.empty')"
    />
    <div
      v-else
      class="overflow-x-auto rounded-box border border-base-300 bg-base-100 shadow-sm"
    >
      <table class="table">
        <thead>
          <tr>
            <th>{{ $t("keys.keyId") }}</th>
            <th>{{ $t("keys.algorithm") }}</th>
            <th>{{ $t("keys.status") }}</th>
            <th>{{ $t("keys.createdAt") }}</th>
            <th>{{ $t("keys.expiresAt") }}</th>
            <th />
          </tr>
        </thead>
        <tbody>
          <tr v-for="key in keysQuery.data.value.items" :key="key.key_id">
            <td>
              <code class="text-xs">{{ key.key_id }}</code>
            </td>
            <td>{{ key.algorithm }}</td>
            <td>
              <span
                class="badge"
                :class="
                  key.revoked_at
                    ? 'badge-error badge-soft'
                    : key.status.toLowerCase().includes('active')
                      ? 'badge-success badge-soft'
                      : 'badge-ghost'
                "
                >{{ formatStatus(key.status) }}</span
              >
            </td>
            <td class="whitespace-nowrap text-sm">
              {{ formatDate(key.created_at) }}
            </td>
            <td class="whitespace-nowrap text-sm">
              {{ formatDate(key.expires_at) }}
            </td>
            <td class="text-right">
              <button
                v-if="!key.revoked_at && key.status.toLowerCase() !== 'revoked'"
                class="btn btn-ghost btn-sm text-error"
                @click="revokeKey = key"
              >
                <Icon icon="material-symbols:block-rounded" />
                {{ $t("keys.revoke") }}
              </button>
              <span v-else class="text-base-content/40">—</span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <ModalDialog
      :open="jwksOpen"
      :title="$t('keys.jwks')"
      wide
      @close="jwksOpen = false"
    >
      <ErrorState
        v-if="jwksQuery.isError.value"
        :message="jwksQuery.error.value?.message"
        @retry="jwksQuery.refetch()"
      />
      <div v-else-if="jwksQuery.isPending.value" class="skeleton h-80 w-full" />
      <div v-else class="relative">
        <button class="btn btn-sm absolute top-2 right-2" @click="copyJwks">
          <Icon icon="material-symbols:content-copy-outline-rounded" />
          {{ $t("common.copy") }}
        </button>
        <pre
          class="max-h-[60vh] overflow-auto rounded-box bg-neutral p-5 pr-28 text-xs text-neutral-content"
          >{{ JSON.stringify(jwksQuery.data.value, null, 2) }}</pre>
      </div>
    </ModalDialog>
    <ConfirmDialog
      :open="Boolean(revokeKey)"
      :title="$t('keys.revokeTitle')"
      :busy="revokeMutation.isPending.value"
      @close="revokeKey = null"
      @confirm="revokeMutation.mutate()"
    />
  </div>
</template>
