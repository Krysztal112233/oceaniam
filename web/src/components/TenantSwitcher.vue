<script setup lang="ts">
import { Icon } from "@iconify/vue/offline";
import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import ModalDialog from "@/components/ModalDialog.vue";
import { api, AUTH_HEADER } from "@/services/api";
import { useTenantStore } from "@/stores/tenant";
import { useToastsStore } from "@/stores/toasts";
import { unwrap } from "@oceaniam/sdk";

const { t } = useI18n();
const tenantStore = useTenantStore();
const toasts = useToastsStore();
const queryClient = useQueryClient();
const createOpen = ref(false);
const comment = ref("");

const tenantsQuery = useQuery({
  queryKey: ["tenants"],
  queryFn: async () =>
    unwrap(
      await api.GET("/tenants", {
        params: { header: AUTH_HEADER, query: { page: 1, per_page: 100 } },
      }),
    ),
});

const tenants = computed(() => tenantsQuery.data.value?.items ?? []);
const current = computed(() =>
  tenants.value.find((item) => item.id === tenantStore.currentId),
);

watch(
  tenants,
  (items) => {
    if (!items.length) tenantStore.select(null);
    else if (!items.some((item) => item.id === tenantStore.currentId))
      tenantStore.select(items[0]?.id ?? null);
  },
  { immediate: true },
);

async function selectTenant(id: string) {
  tenantStore.select(id);
  await queryClient.cancelQueries({
    predicate: (query) => query.queryKey[0] === "tenant",
  });
  await queryClient.invalidateQueries({
    predicate: (query) => query.queryKey[0] === "tenant",
  });
}

const createMutation = useMutation({
  mutationFn: async () =>
    unwrap(
      await api.POST("/tenants", {
        params: { header: AUTH_HEADER },
        body: { comment: comment.value.trim() || null },
      }),
    ),
  onSuccess: async (tenant) => {
    createOpen.value = false;
    comment.value = "";
    await queryClient.invalidateQueries({ queryKey: ["tenants"] });
    await selectTenant(tenant.id);
    toasts.show(t("tenant.create"), "success");
  },
  onError: (error) =>
    toasts.show(
      error instanceof Error ? error.message : t("common.requestFailed"),
      "error",
    ),
});
</script>

<template>
  <div class="dropdown w-full">
    <button
      tabindex="0"
      class="btn btn-ghost h-auto w-full justify-start px-3 py-2"
    >
      <span
        class="bg-primary text-primary-content grid size-9 shrink-0 place-items-center rounded-lg"
      >
        <Icon icon="material-symbols:domain-rounded" class="size-5" />
      </span>
      <span class="min-w-0 flex-1 text-left">
        <span class="text-base-content/55 block text-xs font-normal">{{
          $t("tenant.current")
        }}</span>
        <span class="block truncate text-sm">{{
          current?.id || $t("tenant.none")
        }}</span>
      </span>
      <Icon
        icon="material-symbols:unfold-more-rounded"
        class="size-5 opacity-60"
      />
    </button>
    <ul
      tabindex="0"
      class="menu dropdown-content bg-base-100 rounded-box z-40 mt-2 w-full border border-base-300 p-2 shadow-xl"
    >
      <li v-if="tenantsQuery.isPending.value" class="menu-disabled">
        <span>{{ $t("common.loading") }}</span>
      </li>
      <li v-for="tenant in tenants" :key="tenant.id">
        <button
          :class="{ active: tenant.id === tenantStore.currentId }"
          @click="selectTenant(tenant.id)"
        >
          <Icon icon="material-symbols:domain-rounded" />
          <span class="min-w-0 flex-1">
            <span class="block truncate">{{ tenant.id }}</span>
            <span
              v-if="tenant.comment"
              class="block truncate text-xs opacity-60"
              >{{ tenant.comment }}</span
            >
          </span>
        </button>
      </li>
      <li class="mt-1 border-t border-base-300 pt-1">
        <button class="text-primary" @click="createOpen = true">
          <Icon icon="material-symbols:add-rounded" /> {{ $t("tenant.create") }}
        </button>
      </li>
    </ul>
  </div>

  <ModalDialog
    :open="createOpen"
    :title="$t('tenant.create')"
    @close="createOpen = false"
  >
    <form @submit.prevent="createMutation.mutate()">
      <label class="fieldset">
        <span class="fieldset-legend">{{ $t("tenant.comment") }}</span>
        <textarea
          v-model="comment"
          class="textarea w-full"
          :placeholder="$t('tenant.commentHint')"
        />
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
</template>
