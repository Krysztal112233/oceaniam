<script setup lang="ts">
import { Icon } from "@iconify/vue/offline";
import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { unwrap } from "@oceaniam/sdk";
import { computed, reactive, watch } from "vue";
import { useI18n } from "vue-i18n";

import ErrorState from "@/components/ErrorState.vue";
import { api, APPLICATION_AUTH_HEADER, AUTH_HEADER } from "@/services/api";
import { useToastsStore } from "@/stores/toasts";

const props = defineProps<{ tenantId: string; applicationId: string }>();
const { t } = useI18n();
const toasts = useToastsStore();
const queryClient = useQueryClient();
const queryKey = computed(() => [
  "tenant",
  props.tenantId,
  "application",
  props.applicationId,
  "configuration",
]);
const form = reactive({ issuer: "", audience: "", registrationEnabled: false });

const configurationQuery = useQuery({
  queryKey,
  queryFn: async () =>
    unwrap(
      await api.GET(
        "/tenants/{tenant_id}/applications/{application_id}/configuration",
        {
          params: {
            header: APPLICATION_AUTH_HEADER,
            path: {
              tenant_id: props.tenantId,
              application_id: props.applicationId,
            },
          },
        },
      ),
    ),
});

watch(
  () => configurationQuery.data.value,
  (data) => {
    if (!data) return;
    form.issuer = data.configuration.auth.token.issuer;
    form.audience = data.configuration.auth.token.audience.join("\n");
    form.registrationEnabled = data.configuration.registration.enabled;
  },
  { immediate: true },
);

const updateMutation = useMutation({
  mutationFn: async () =>
    unwrap(
      await api.PATCH(
        "/tenants/{tenant_id}/applications/{application_id}/configuration",
        {
          params: {
            header: AUTH_HEADER,
            path: {
              tenant_id: props.tenantId,
              application_id: props.applicationId,
            },
          },
          body: {
            auth: {
              token: {
                issuer: form.issuer.trim(),
                audience: form.audience
                  .split(/[\n,]/)
                  .map((value) => value.trim())
                  .filter(Boolean),
              },
            },
            registration: { enabled: form.registrationEnabled },
          },
        },
      ),
    ),
  onSuccess: async () => {
    await queryClient.invalidateQueries({ queryKey: queryKey.value });
    toasts.show(t("common.saved"), "success");
  },
  onError: (error) =>
    toasts.show(
      error instanceof Error ? error.message : t("common.requestFailed"),
      "error",
    ),
});
</script>

<template>
  <ErrorState
    v-if="configurationQuery.isError.value"
    :message="configurationQuery.error.value?.message"
    @retry="configurationQuery.refetch()"
  />
  <div
    v-else-if="configurationQuery.isPending.value"
    class="grid gap-4 md:grid-cols-2"
  >
    <div class="skeleton h-56" />
    <div class="skeleton h-56" />
  </div>
  <form
    v-else
    class="grid gap-5 lg:grid-cols-[minmax(0,2fr)_minmax(18rem,1fr)]"
    @submit.prevent="updateMutation.mutate()"
  >
    <section class="rounded-box border border-base-300 p-4 sm:p-5">
      <h3 class="flex items-center gap-2 font-semibold">
        <Icon icon="material-symbols:token-outline-rounded" />
        {{ $t("applications.token") }}
      </h3>
      <label class="fieldset mt-3">
        <span class="fieldset-legend">{{ $t("applications.issuer") }}</span>
        <input v-model="form.issuer" class="input w-full" required />
      </label>
      <label class="fieldset mt-2">
        <span class="fieldset-legend">{{ $t("applications.audience") }}</span>
        <textarea
          v-model="form.audience"
          class="textarea min-h-28 w-full font-mono text-sm"
          :placeholder="$t('applications.audienceHint')"
        />
      </label>
      <div
        class="mt-5 flex flex-col gap-4 border-t border-base-300 pt-5 sm:flex-row sm:items-center sm:justify-between"
      >
        <label class="flex cursor-pointer items-center gap-3">
          <input
            v-model="form.registrationEnabled"
            type="checkbox"
            class="toggle toggle-primary"
          />
          <span class="font-medium">{{ $t("applications.registration") }}</span>
        </label>
        <button
          type="submit"
          class="btn btn-primary min-w-24 max-sm:w-full"
          :disabled="updateMutation.isPending.value"
        >
          <span
            v-if="updateMutation.isPending.value"
            class="loading loading-spinner loading-sm"
          />
          {{ $t("common.save") }}
        </button>
      </div>
    </section>

    <section class="rounded-box border border-base-300 bg-base-200 p-4 sm:p-5">
      <h3 class="flex items-center gap-2 font-semibold">
        <Icon icon="material-symbols:password-rounded" />
        {{ $t("applications.argon2") }}
      </h3>
      <dl class="mt-4 space-y-3 text-sm">
        <div class="flex justify-between gap-4">
          <dt class="text-base-content/60">
            {{ $t("applications.memoryCost") }}
          </dt>
          <dd class="font-mono">
            {{
              configurationQuery.data.value?.configuration.auth.password.argon2
                .m_cost
            }}
          </dd>
        </div>
        <div class="flex justify-between gap-4">
          <dt class="text-base-content/60">
            {{ $t("applications.timeCost") }}
          </dt>
          <dd class="font-mono">
            {{
              configurationQuery.data.value?.configuration.auth.password.argon2
                .t_cost
            }}
          </dd>
        </div>
        <div class="flex justify-between gap-4">
          <dt class="text-base-content/60">
            {{ $t("applications.parallelism") }}
          </dt>
          <dd class="font-mono">
            {{
              configurationQuery.data.value?.configuration.auth.password.argon2
                .p_cost
            }}
          </dd>
        </div>
      </dl>
      <p class="text-base-content/50 mt-5 text-xs">
        {{ $t("applications.passwordManaged") }}
      </p>
    </section>
  </form>
</template>
