<script setup lang="ts">
import { Icon } from "@iconify/vue/offline";
import { useMutation, useQuery } from "@tanstack/vue-query";
import type { components } from "@oceaniam/sdk";
import { unwrap } from "@oceaniam/sdk";
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

import ConfirmDialog from "@/components/ConfirmDialog.vue";
import { api, AUTH_HEADER } from "@/services/api";
import { useTenantStore } from "@/stores/tenant";
import { useToastsStore } from "@/stores/toasts";

type Secret = components["schemas"]["SecretVO"];

const props = defineProps<{ secret: Secret; expanded: boolean }>();
const emit = defineEmits<{ toggle: []; changed: []; deleted: [] }>();
const tenant = useTenantStore();
const toasts = useToastsStore();
const { t, locale } = useI18n();
const tab = ref<"overview" | "bindings" | "settings">("bindings");
const bindApplicationId = ref("");
const deleteOpen = ref(false);
const unbindId = ref<string | null>(null);

const applicationsQuery = useQuery({
  queryKey: computed(() => [
    "tenant",
    tenant.currentId,
    "applications",
    "bindings",
  ]),
  enabled: computed(() => props.expanded && Boolean(tenant.currentId)),
  queryFn: async () => {
    if (!tenant.currentId) throw new Error("No tenant selected");
    return unwrap(
      await api.GET("/tenants/{tenant_id}/applications", {
        params: {
          header: AUTH_HEADER,
          path: { tenant_id: tenant.currentId },
          query: { page: 1, per_page: 100 },
        },
      }),
    );
  },
});

const availableApplications = computed(() =>
  (applicationsQuery.data.value?.items ?? []).filter(
    (application) => !props.secret.application_ids.includes(application.id),
  ),
);

const bindMutation = useMutation({
  mutationFn: async () => {
    if (!bindApplicationId.value) throw new Error("Select an application");
    return unwrap(
      await api.POST("/secrets/{secret_id}/bindings", {
        params: { header: AUTH_HEADER, path: { secret_id: props.secret.id } },
        body: { application_id: bindApplicationId.value },
      }),
    );
  },
  onSuccess: () => {
    bindApplicationId.value = "";
    emit("changed");
    toasts.show(t("common.saved"), "success");
  },
  onError: (error) =>
    toasts.show(
      error instanceof Error ? error.message : t("common.requestFailed"),
      "error",
    ),
});

const unbindMutation = useMutation({
  mutationFn: async () => {
    if (!unbindId.value) throw new Error("No application selected");
    return unwrap(
      await api.DELETE("/secrets/{secret_id}/bindings/{application_id}", {
        params: {
          header: AUTH_HEADER,
          path: { secret_id: props.secret.id, application_id: unbindId.value },
        },
      }),
    );
  },
  onSuccess: () => {
    unbindId.value = null;
    emit("changed");
    toasts.show(t("common.saved"), "success");
  },
  onError: (error) =>
    toasts.show(
      error instanceof Error ? error.message : t("common.requestFailed"),
      "error",
    ),
});

const deleteMutation = useMutation({
  mutationFn: async () =>
    unwrap(
      await api.DELETE("/secrets/{secret_id}", {
        params: { header: AUTH_HEADER, path: { secret_id: props.secret.id } },
      }),
    ),
  onSuccess: () => {
    deleteOpen.value = false;
    emit("deleted");
    toasts.show(t("common.deleted"), "success");
  },
  onError: (error) =>
    toasts.show(
      error instanceof Error ? error.message : t("common.requestFailed"),
      "error",
    ),
});

function formatDate(value: string | null | undefined) {
  if (!value) return t("common.never");
  return new Intl.DateTimeFormat(locale.value, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}
</script>

<template>
  <article
    class="collapse-arrow collapse border border-base-300 bg-base-100 shadow-sm"
    :class="expanded ? 'collapse-open' : 'collapse-close'"
  >
    <button
      class="collapse-title flex min-h-24 items-center gap-4 pr-12 text-left"
      @click="emit('toggle')"
    >
      <span
        class="bg-secondary/10 text-secondary grid size-12 shrink-0 place-items-center rounded-xl"
        ><Icon icon="material-symbols:key-outline-rounded" class="size-7"
      /></span>
      <span class="min-w-0 flex-1"
        ><span class="block truncate text-lg font-semibold">{{
          secret.id
        }}</span
        ><code class="text-base-content/55 mt-1 block truncate text-sm">{{
          secret.secret
        }}</code></span
      >
      <span class="badge badge-outline max-md:hidden"
        >{{ secret.application_ids.length }} {{ $t("secrets.bindings") }}</span
      >
    </button>
    <div v-if="expanded" class="collapse-content px-0 pb-0">
      <div class="tabs tabs-border border-y border-base-300 px-4 sm:px-6">
        <button
          class="tab"
          :class="{ 'tab-active': tab === 'overview' }"
          @click="tab = 'overview'"
        >
          {{ $t("applications.overview") }}
        </button>
        <button
          class="tab"
          :class="{ 'tab-active': tab === 'bindings' }"
          @click="tab = 'bindings'"
        >
          {{ $t("secrets.bindings") }}
        </button>
        <button
          class="tab"
          :class="{ 'tab-active': tab === 'settings' }"
          @click="tab = 'settings'"
        >
          {{ $t("applications.settings") }}
        </button>
      </div>
      <div class="p-4 sm:p-6">
        <dl v-if="tab === 'overview'" class="grid gap-4 sm:grid-cols-2">
          <div class="rounded-box bg-base-200 p-4">
            <dt class="text-base-content/55 text-xs uppercase">ID</dt>
            <dd class="mt-1 break-all font-mono">
              {{ secret.id }}
            </dd>
          </div>
          <div class="rounded-box bg-base-200 p-4">
            <dt class="text-base-content/55 text-xs uppercase">
              {{ $t("secrets.createdAt") }}
            </dt>
            <dd class="mt-1">
              {{ formatDate(secret.created_at) }}
            </dd>
          </div>
          <div class="rounded-box bg-base-200 p-4 sm:col-span-2">
            <dt class="text-base-content/55 text-xs uppercase">
              {{ $t("secrets.secret") }}
            </dt>
            <dd class="mt-1 break-all font-mono">
              {{ secret.secret }}
            </dd>
          </div>
        </dl>
        <section v-else-if="tab === 'bindings'" class="space-y-4">
          <div class="flex flex-col gap-2 sm:flex-row">
            <select
              v-model="bindApplicationId"
              class="select min-w-0 flex-1"
              :disabled="!tenant.currentId || applicationsQuery.isPending.value"
            >
              <option value="">
                {{ tenant.currentId ? $t("secrets.bind") : $t("tenant.none") }}
              </option>
              <option
                v-for="application in availableApplications"
                :key="application.id"
                :value="application.id"
              >
                {{ application.id
                }}{{ application.comment ? ` · ${application.comment}` : "" }}
              </option>
            </select>
            <button
              class="btn btn-primary"
              :disabled="!bindApplicationId || bindMutation.isPending.value"
              @click="bindMutation.mutate()"
            >
              <Icon icon="material-symbols:add-link-rounded" />
              {{ $t("secrets.bind") }}
            </button>
          </div>
          <div
            v-if="!secret.application_ids.length"
            class="text-base-content/55 py-6 text-center"
          >
            {{ $t("common.noData") }}
          </div>
          <div v-else class="flex flex-wrap gap-2">
            <span
              v-for="id in secret.application_ids"
              :key="id"
              class="badge badge-lg gap-2 py-4"
              ><Icon icon="material-symbols:apps-rounded" />{{ id
              }}<button
                class="btn btn-ghost btn-circle btn-xs"
                :aria-label="$t('secrets.unbind')"
                @click="unbindId = id"
              >
                ✕
              </button></span
            >
          </div>
        </section>
        <section
          v-else
          class="rounded-box border border-error/30 bg-error/5 p-4"
        >
          <h3 class="font-semibold text-error">
            {{ $t("secrets.deleteTitle") }}
          </h3>
          <p class="text-base-content/60 mt-2 text-sm">
            {{ $t("common.dangerConfirm") }}
          </p>
          <button
            class="btn btn-error btn-outline btn-sm mt-4"
            @click="deleteOpen = true"
          >
            <Icon icon="material-symbols:delete-outline-rounded" />
            {{ $t("common.delete") }}
          </button>
        </section>
      </div>
    </div>
  </article>

  <ConfirmDialog
    :open="Boolean(unbindId)"
    :title="$t('secrets.unbind')"
    :busy="unbindMutation.isPending.value"
    @close="unbindId = null"
    @confirm="unbindMutation.mutate()"
  />
  <ConfirmDialog
    :open="deleteOpen"
    :title="$t('secrets.deleteTitle')"
    :busy="deleteMutation.isPending.value"
    @close="deleteOpen = false"
    @confirm="deleteMutation.mutate()"
  />
</template>
