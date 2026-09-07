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
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import EmptyState from "@/components/EmptyState.vue";
import ErrorState from "@/components/ErrorState.vue";
import ModalDialog from "@/components/ModalDialog.vue";
import PageHeader from "@/components/PageHeader.vue";
import PaginationBar from "@/components/PaginationBar.vue";
import SecretCard from "@/components/secrets/SecretCard.vue";
import { api, AUTH_HEADER } from "@/services/api";
import { useTenantStore } from "@/stores/tenant";
import { useToastsStore } from "@/stores/toasts";

type Secret = components["schemas"]["SecretVO"];

const { t } = useI18n();
const tenant = useTenantStore();
const toasts = useToastsStore();
const queryClient = useQueryClient();
const page = ref(1);
const expandedId = ref<string | null>(null);
const createOpen = ref(false);
const created = ref<Secret | null>(null);

watch(
  () => tenant.revision,
  () => {
    expandedId.value = null;
  },
);

const secretsQuery = useQuery({
  queryKey: ["secrets", page],
  placeholderData: keepPreviousData,
  queryFn: async () =>
    unwrap(
      await api.GET("/secrets", {
        params: {
          header: AUTH_HEADER,
          query: { page: page.value, per_page: 20 },
        },
      }),
    ),
});

const createMutation = useMutation({
  mutationFn: async () =>
    unwrap(await api.POST("/secrets", { params: { header: AUTH_HEADER } })),
  onSuccess: async (secret) => {
    created.value = secret;
    await queryClient.invalidateQueries({ queryKey: ["secrets"] });
  },
  onError: (error) =>
    toasts.show(
      error instanceof Error ? error.message : t("common.requestFailed"),
      "error",
    ),
});

function openCreate() {
  created.value = null;
  createOpen.value = true;
}

function closeCreate() {
  if (created.value) expandedId.value = created.value.id;
  createOpen.value = false;
  created.value = null;
}

async function copySecret(value: string) {
  await navigator.clipboard.writeText(value);
  toasts.show(t("common.copied"), "success");
}
</script>

<template>
  <div class="page-shell">
    <PageHeader :title="$t('secrets.title')" :subtitle="$t('secrets.subtitle')">
      <template #actions>
        <button class="btn btn-primary" @click="openCreate">
          <Icon icon="material-symbols:add-rounded" />
          {{ $t("secrets.create") }}
        </button>
      </template>
    </PageHeader>

    <ErrorState
      v-if="secretsQuery.isError.value"
      :message="secretsQuery.error.value?.message"
      @retry="secretsQuery.refetch()"
    />
    <div v-else-if="secretsQuery.isPending.value" class="space-y-3">
      <div v-for="index in 4" :key="index" class="skeleton h-24 w-full" />
    </div>
    <EmptyState
      v-else-if="!secretsQuery.data.value?.items.length"
      icon="material-symbols:key-outline-rounded"
      :title="$t('secrets.empty')"
    >
      <button class="btn btn-primary btn-sm" @click="openCreate">
        {{ $t("secrets.create") }}
      </button>
    </EmptyState>
    <div v-else class="space-y-3">
      <SecretCard
        v-for="secret in secretsQuery.data.value.items"
        :key="secret.id"
        :secret="secret"
        :expanded="expandedId === secret.id"
        @toggle="expandedId = expandedId === secret.id ? null : secret.id"
        @changed="queryClient.invalidateQueries({ queryKey: ['secrets'] })"
        @deleted="
          expandedId = null;
          queryClient.invalidateQueries({ queryKey: ['secrets'] });
        "
      />
      <PaginationBar
        :page="page"
        :total="secretsQuery.data.value.page_info.total"
        :has-next="secretsQuery.data.value.page_info.has_next"
        @change="
          page = $event;
          expandedId = null;
        "
      />
    </div>

    <ModalDialog
      :open="createOpen"
      :title="$t('secrets.create')"
      :close-on-backdrop="!created"
      @close="closeCreate"
    >
      <div v-if="created" class="space-y-4">
        <div class="alert alert-warning alert-soft">
          <Icon icon="material-symbols:warning-outline-rounded" /><span>{{
            $t("secrets.revealWarning")
          }}</span>
        </div>
        <div>
          <div class="text-base-content/60 text-xs font-semibold uppercase">
            ID
          </div>
          <code class="mt-1 block break-all">{{ created.id }}</code>
        </div>
        <div>
          <div class="text-base-content/60 text-xs font-semibold uppercase">
            {{ $t("secrets.secret") }}
          </div>
          <div class="join mt-1 flex">
            <input
              class="input join-item min-w-0 flex-1 font-mono"
              readonly
              :value="created.secret"
            /><button class="btn join-item" @click="copySecret(created.secret)">
              <Icon icon="material-symbols:content-copy-outline-rounded" />
              {{ $t("common.copy") }}
            </button>
          </div>
        </div>
        <div class="modal-action">
          <button class="btn btn-primary" @click="closeCreate">
            {{ $t("common.close") }}
          </button>
        </div>
      </div>
      <div v-else>
        <p>{{ $t("secrets.revealWarning") }}</p>
        <div class="modal-action">
          <button
            class="btn"
            :disabled="createMutation.isPending.value"
            @click="closeCreate"
          >
            {{ $t("common.cancel") }}</button
          ><button
            class="btn btn-primary"
            :disabled="createMutation.isPending.value"
            @click="createMutation.mutate()"
          >
            <span
              v-if="createMutation.isPending.value"
              class="loading loading-spinner loading-sm"
            />{{ $t("common.create") }}
          </button>
        </div>
      </div>
    </ModalDialog>
  </div>
</template>
