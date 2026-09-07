<script setup lang="ts">
import { Icon } from "@iconify/vue/offline";
import { useMutation, useQueryClient } from "@tanstack/vue-query";
import type { components } from "@oceaniam/sdk";
import { unwrap } from "@oceaniam/sdk";
import { ref } from "vue";
import { useI18n } from "vue-i18n";

import ApplicationConfigurationPanel from "@/components/applications/ApplicationConfigurationPanel.vue";
import ApplicationOverviewPanel from "@/components/applications/ApplicationOverviewPanel.vue";
import ApplicationUsersPanel from "@/components/applications/ApplicationUsersPanel.vue";
import ConfirmDialog from "@/components/ConfirmDialog.vue";
import { api, AUTH_HEADER } from "@/services/api";
import { useToastsStore } from "@/stores/toasts";

type Application = components["schemas"]["ApplicationVO"];
type Tab = "overview" | "users" | "configuration" | "settings";

const props = defineProps<{ application: Application; expanded: boolean }>();
const emit = defineEmits<{ toggle: []; changed: []; deleted: [] }>();
const { t } = useI18n();
const toasts = useToastsStore();
const queryClient = useQueryClient();
const tab = ref<Tab>("overview");
const comment = ref(props.application.comment ?? "");
const deleteOpen = ref(false);

const patchMutation = useMutation({
  mutationFn: async () =>
    unwrap(
      await api.PATCH("/tenants/{tenant_id}/applications/{application_id}", {
        params: {
          header: AUTH_HEADER,
          path: {
            tenant_id: props.application.tenant_id,
            application_id: props.application.id,
          },
        },
        body: { comment: comment.value.trim() || null },
      }),
    ),
  onSuccess: () => {
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
      await api.DELETE("/tenants/{tenant_id}/applications/{application_id}", {
        params: {
          header: AUTH_HEADER,
          path: {
            tenant_id: props.application.tenant_id,
            application_id: props.application.id,
          },
        },
      }),
    ),
  onSuccess: () => {
    deleteOpen.value = false;
    queryClient.removeQueries({
      queryKey: [
        "tenant",
        props.application.tenant_id,
        "application",
        props.application.id,
      ],
    });
    emit("deleted");
    toasts.show(t("common.deleted"), "success");
  },
  onError: (error) =>
    toasts.show(
      error instanceof Error ? error.message : t("common.requestFailed"),
      "error",
    ),
});

const tabs: Array<{ value: Tab; label: string; icon: string }> = [
  {
    value: "overview",
    label: "applications.overview",
    icon: "material-symbols:space-dashboard-outline-rounded",
  },
  {
    value: "users",
    label: "applications.users",
    icon: "material-symbols:group-outline-rounded",
  },
  {
    value: "configuration",
    label: "applications.configuration",
    icon: "material-symbols:tune-rounded",
  },
  {
    value: "settings",
    label: "applications.settings",
    icon: "material-symbols:settings-outline-rounded",
  },
];
</script>

<template>
  <article
    class="collapse-arrow collapse border border-base-300 bg-base-100 shadow-sm"
    :class="{ 'collapse-open': expanded, 'collapse-close': !expanded }"
  >
    <button
      class="collapse-title flex min-h-24 items-center gap-4 pr-12 text-left"
      type="button"
      @click="emit('toggle')"
    >
      <span
        class="bg-primary/10 text-primary grid size-12 shrink-0 place-items-center rounded-xl"
      >
        <Icon icon="material-symbols:apps-rounded" class="size-7" />
      </span>
      <span class="min-w-0 flex-1">
        <span class="block truncate text-lg font-semibold">{{
          application.id
        }}</span>
        <span class="text-base-content/55 mt-1 block truncate text-sm">{{
          application.comment || $t("common.noData")
        }}</span>
      </span>
      <span class="badge badge-ghost max-md:hidden">{{
        application.tenant_id
      }}</span>
    </button>

    <div v-if="expanded" class="collapse-content px-0 pb-0">
      <div
        class="tabs tabs-border overflow-x-auto border-y border-base-300 px-4 sm:px-6"
        role="tablist"
      >
        <button
          v-for="item in tabs"
          :key="item.value"
          class="tab gap-2"
          :class="{ 'tab-active': tab === item.value }"
          role="tab"
          @click="tab = item.value"
        >
          <Icon :icon="item.icon" class="size-4" /> {{ $t(item.label) }}
        </button>
      </div>
      <div class="p-4 sm:p-6">
        <ApplicationOverviewPanel
          v-if="tab === 'overview'"
          :tenant-id="application.tenant_id"
          :application-id="application.id"
        />
        <ApplicationUsersPanel
          v-else-if="tab === 'users'"
          :tenant-id="application.tenant_id"
          :application-id="application.id"
        />
        <ApplicationConfigurationPanel
          v-else-if="tab === 'configuration'"
          :tenant-id="application.tenant_id"
          :application-id="application.id"
        />
        <section v-else class="grid gap-6 lg:grid-cols-2">
          <form
            class="rounded-box border border-base-300 p-4"
            @submit.prevent="patchMutation.mutate()"
          >
            <h3 class="font-semibold">
              {{ $t("applications.updateComment") }}
            </h3>
            <textarea v-model="comment" class="textarea mt-3 w-full" rows="3" />
            <button
              class="btn btn-primary btn-sm mt-3"
              :disabled="patchMutation.isPending.value"
            >
              <span
                v-if="patchMutation.isPending.value"
                class="loading loading-spinner loading-xs"
              />
              {{ $t("common.save") }}
            </button>
          </form>
          <div class="rounded-box border border-error/30 bg-error/5 p-4">
            <h3 class="font-semibold text-error">
              {{ $t("applications.deleteTitle") }}
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
          </div>
        </section>
      </div>
    </div>
  </article>

  <ConfirmDialog
    :open="deleteOpen"
    :title="$t('applications.deleteTitle')"
    :busy="deleteMutation.isPending.value"
    @close="deleteOpen = false"
    @confirm="deleteMutation.mutate()"
  />
</template>
