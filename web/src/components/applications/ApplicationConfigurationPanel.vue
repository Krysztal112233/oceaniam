<script setup lang="ts">
import { Icon } from "@iconify/vue/offline";
import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { unwrap } from "@oceaniam/sdk";
import { computed, nextTick, reactive, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import ErrorState from "@/components/ErrorState.vue";
import {
  applicationConfigurationPatch,
  applicationConfigurationToForm,
  MAX_DEVELOPMENT_ACCOUNT_TTL_SECONDS,
  validateApplicationConfiguration,
  type ApplicationConfigurationForm,
  type ConfigurationError,
} from "@/features/applications/configuration";
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
const initialForm: ApplicationConfigurationForm = {
  issuer: "",
  audience: "",
  registrationEnabled: false,
  developmentAccountsEnabled: true,
  defaultTtlSeconds: "3600",
  maxTtlSeconds: "86400",
  allowInsecureLoopbackRedirectUris: false,
};
const form = reactive<ApplicationConfigurationForm>({ ...initialForm });
const savedForm = ref<ApplicationConfigurationForm>({ ...initialForm });
const formElement = ref<HTMLFormElement | null>(null);
const validationAttempted = ref(false);
const idPrefix = `application-${props.applicationId}-configuration`;

const isDirty = computed(
  () => JSON.stringify(form) !== JSON.stringify(savedForm.value),
);
const validationErrors = computed(() => validateApplicationConfiguration(form));

function hasError(...errors: ConfigurationError[]) {
  return errors.some((error) => validationErrors.value.includes(error));
}

function showError(error: ConfigurationError) {
  return validationAttempted.value && hasError(error);
}

function applyConfiguration(
  configuration: Parameters<typeof applicationConfigurationToForm>[0],
) {
  const next = applicationConfigurationToForm(configuration);
  savedForm.value = next;
  Object.assign(form, next);
  validationAttempted.value = false;
}

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
    if (data && !isDirty.value) applyConfiguration(data.configuration);
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
          body: applicationConfigurationPatch(form),
        },
      ),
    ),
  onSuccess: async () => {
    savedForm.value = { ...form };
    validationAttempted.value = false;
    await queryClient.invalidateQueries({ queryKey: queryKey.value });
    toasts.show(t("common.saved"), "success");
  },
  onError: () => toasts.show(t("common.saveFailed"), "error"),
});

function resetForm() {
  Object.assign(form, savedForm.value);
  validationAttempted.value = false;
}

async function submitForm() {
  validationAttempted.value = true;
  if (validationErrors.value.length > 0) {
    await nextTick();
    formElement.value
      ?.querySelector<HTMLElement>('[aria-invalid="true"]')
      ?.focus();
    return;
  }
  updateMutation.mutate();
}
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
    ref="formElement"
    :aria-label="$t('applications.configuration')"
    novalidate
    @submit.prevent="submitForm"
  >
    <fieldset class="min-w-0" :disabled="updateMutation.isPending.value">
      <div class="grid gap-5 xl:grid-cols-[minmax(0,2fr)_minmax(18rem,1fr)]">
        <div class="space-y-5">
          <section class="rounded-box border border-base-300 p-4 sm:p-5">
            <h3 class="flex items-center gap-2 font-semibold">
              <Icon icon="material-symbols:token-outline-rounded" />
              {{ $t("applications.token") }}
            </h3>
            <p class="text-base-content/60 mt-2 text-sm">
              {{ $t("applications.tokenDescription") }}
            </p>
            <div class="fieldset mt-3">
              <label class="fieldset-legend" :for="`${idPrefix}-issuer`">
                {{ $t("applications.issuer") }}
              </label>
              <input
                :id="`${idPrefix}-issuer`"
                v-model="form.issuer"
                class="input w-full"
                :class="{ 'input-error': showError('issuerRequired') }"
                required
                :aria-invalid="hasError('issuerRequired')"
                :aria-describedby="
                  showError('issuerRequired')
                    ? `${idPrefix}-issuer-error`
                    : undefined
                "
              />
              <p
                v-if="showError('issuerRequired')"
                :id="`${idPrefix}-issuer-error`"
                class="mt-1 text-sm text-error"
              >
                {{ $t("applications.issuerRequired") }}
              </p>
            </div>
            <div class="fieldset mt-2">
              <label class="fieldset-legend" :for="`${idPrefix}-audience`">
                {{ $t("applications.audience") }}
                <span class="badge badge-ghost badge-sm">
                  {{ $t("common.optional") }}
                </span>
              </label>
              <textarea
                :id="`${idPrefix}-audience`"
                v-model="form.audience"
                class="textarea min-h-28 w-full font-mono text-sm"
                :aria-describedby="`${idPrefix}-audience-hint`"
                :placeholder="$t('applications.audienceHint')"
              />
              <p
                :id="`${idPrefix}-audience-hint`"
                class="text-base-content/55 mt-1 text-xs"
              >
                {{ $t("applications.audienceHint") }}
              </p>
            </div>
          </section>

          <section class="rounded-box border border-base-300 p-4 sm:p-5">
            <h3 class="flex items-center gap-2 font-semibold">
              <Icon icon="material-symbols:group-add-outline-rounded" />
              {{ $t("applications.accountAccess") }}
            </h3>
            <div
              class="mt-4 flex items-start justify-between gap-4 border-b border-base-300 pb-5"
            >
              <div>
                <label class="font-medium" :for="`${idPrefix}-registration`">
                  {{ $t("applications.registration") }}
                </label>
                <p
                  :id="`${idPrefix}-registration-hint`"
                  class="text-base-content/55 mt-1 text-sm"
                >
                  {{ $t("applications.registrationDescription") }}
                </p>
              </div>
              <input
                :id="`${idPrefix}-registration`"
                v-model="form.registrationEnabled"
                type="checkbox"
                class="toggle toggle-primary mt-1 shrink-0"
                :aria-describedby="`${idPrefix}-registration-hint`"
              />
            </div>

            <div class="mt-5 flex items-start justify-between gap-4">
              <div>
                <label
                  class="font-medium"
                  :for="`${idPrefix}-development-enabled`"
                >
                  {{ $t("applications.developmentAccounts") }}
                </label>
                <p
                  :id="`${idPrefix}-development-hint`"
                  class="text-base-content/55 mt-1 text-sm"
                >
                  {{ $t("applications.developmentAccountsDescription") }}
                </p>
              </div>
              <input
                :id="`${idPrefix}-development-enabled`"
                v-model="form.developmentAccountsEnabled"
                type="checkbox"
                class="toggle toggle-primary mt-1 shrink-0"
                :aria-describedby="`${idPrefix}-development-hint`"
              />
            </div>
            <div class="mt-4 grid gap-4 sm:grid-cols-2">
              <div class="fieldset">
                <label class="fieldset-legend" :for="`${idPrefix}-default-ttl`">
                  {{ $t("applications.defaultTtl") }}
                </label>
                <input
                  :id="`${idPrefix}-default-ttl`"
                  v-model="form.defaultTtlSeconds"
                  type="number"
                  min="1"
                  :max="MAX_DEVELOPMENT_ACCOUNT_TTL_SECONDS"
                  step="1"
                  inputmode="numeric"
                  class="input w-full"
                  :class="{ 'input-error': showError('defaultTtlInvalid') }"
                  :aria-invalid="hasError('defaultTtlInvalid')"
                  :aria-describedby="
                    showError('defaultTtlInvalid')
                      ? `${idPrefix}-default-ttl-error`
                      : `${idPrefix}-ttl-hint`
                  "
                />
                <p
                  v-if="showError('defaultTtlInvalid')"
                  :id="`${idPrefix}-default-ttl-error`"
                  class="mt-1 text-sm text-error"
                >
                  {{ $t("applications.ttlInvalid") }}
                </p>
              </div>
              <div class="fieldset">
                <label class="fieldset-legend" :for="`${idPrefix}-max-ttl`">
                  {{ $t("applications.maxTtl") }}
                </label>
                <input
                  :id="`${idPrefix}-max-ttl`"
                  v-model="form.maxTtlSeconds"
                  type="number"
                  min="1"
                  :max="MAX_DEVELOPMENT_ACCOUNT_TTL_SECONDS"
                  step="1"
                  inputmode="numeric"
                  class="input w-full"
                  :class="{
                    'input-error':
                      showError('maxTtlInvalid') || showError('ttlOrder'),
                  }"
                  :aria-invalid="hasError('maxTtlInvalid', 'ttlOrder')"
                  :aria-describedby="
                    showError('maxTtlInvalid') || showError('ttlOrder')
                      ? `${idPrefix}-max-ttl-error`
                      : `${idPrefix}-ttl-hint`
                  "
                />
                <p
                  v-if="showError('maxTtlInvalid') || showError('ttlOrder')"
                  :id="`${idPrefix}-max-ttl-error`"
                  class="mt-1 text-sm text-error"
                >
                  {{
                    showError("maxTtlInvalid")
                      ? $t("applications.ttlInvalid")
                      : $t("applications.ttlOrder")
                  }}
                </p>
              </div>
            </div>
            <p
              :id="`${idPrefix}-ttl-hint`"
              class="text-base-content/55 mt-2 text-xs"
            >
              {{
                $t("applications.ttlHint", {
                  max: MAX_DEVELOPMENT_ACCOUNT_TTL_SECONDS,
                })
              }}
            </p>
          </section>
        </div>

        <div class="space-y-5">
          <section class="rounded-box border border-base-300 p-4 sm:p-5">
            <h3 class="flex items-center gap-2 font-semibold">
              <Icon icon="material-symbols:passkey-rounded" />
              {{ $t("applications.oidc") }}
            </h3>
            <p class="text-base-content/60 mt-2 text-sm">
              {{ $t("applications.oidcDescription") }}
            </p>
            <div class="mt-4 flex items-start justify-between gap-4">
              <div>
                <label
                  class="font-medium"
                  :for="`${idPrefix}-insecure-loopback`"
                >
                  {{ $t("applications.allowInsecureLoopback") }}
                </label>
                <p
                  :id="`${idPrefix}-insecure-loopback-hint`"
                  class="text-base-content/55 mt-1 text-sm"
                >
                  {{ $t("applications.allowInsecureLoopbackDescription") }}
                </p>
              </div>
              <input
                :id="`${idPrefix}-insecure-loopback`"
                v-model="form.allowInsecureLoopbackRedirectUris"
                type="checkbox"
                class="toggle toggle-warning mt-1 shrink-0"
                :aria-describedby="`${idPrefix}-insecure-loopback-hint`"
              />
            </div>
            <div
              v-if="form.allowInsecureLoopbackRedirectUris"
              class="alert alert-warning alert-soft mt-4 text-sm"
            >
              <Icon
                icon="material-symbols:warning-outline-rounded"
                class="size-5 shrink-0"
              />
              <span>{{ $t("applications.insecureLoopbackWarning") }}</span>
            </div>
          </section>

          <section
            class="rounded-box border border-base-300 bg-base-200 p-4 sm:p-5"
          >
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
                    configurationQuery.data.value?.configuration.auth.password
                      .argon2.m_cost
                  }}
                </dd>
              </div>
              <div class="flex justify-between gap-4">
                <dt class="text-base-content/60">
                  {{ $t("applications.timeCost") }}
                </dt>
                <dd class="font-mono">
                  {{
                    configurationQuery.data.value?.configuration.auth.password
                      .argon2.t_cost
                  }}
                </dd>
              </div>
              <div class="flex justify-between gap-4">
                <dt class="text-base-content/60">
                  {{ $t("applications.parallelism") }}
                </dt>
                <dd class="font-mono">
                  {{
                    configurationQuery.data.value?.configuration.auth.password
                      .argon2.p_cost
                  }}
                </dd>
              </div>
            </dl>
            <p class="text-base-content/50 mt-5 text-xs">
              {{ $t("applications.passwordManaged") }}
            </p>
          </section>
        </div>
      </div>

      <div
        class="mt-5 flex flex-col gap-4 rounded-box border border-base-300 bg-base-100 p-4 sm:flex-row sm:items-center sm:justify-between"
      >
        <p
          class="flex items-center gap-2 text-sm"
          :class="isDirty ? 'text-warning' : 'text-base-content/55'"
          aria-live="polite"
        >
          <Icon
            :icon="
              isDirty
                ? 'material-symbols:edit-outline-rounded'
                : 'material-symbols:check-circle-outline-rounded'
            "
            class="size-5"
          />
          {{
            isDirty ? $t("common.unsavedChanges") : $t("common.allChangesSaved")
          }}
        </p>
        <div class="flex gap-2 max-sm:flex-col-reverse">
          <button
            type="button"
            class="btn btn-ghost"
            :disabled="!isDirty || updateMutation.isPending.value"
            @click="resetForm"
          >
            {{ $t("common.reset") }}
          </button>
          <button
            type="submit"
            class="btn btn-primary min-w-28"
            :disabled="!isDirty || updateMutation.isPending.value"
          >
            <span
              v-if="updateMutation.isPending.value"
              class="loading loading-spinner loading-sm"
            />
            {{
              updateMutation.isPending.value
                ? $t("common.saving")
                : $t("common.save")
            }}
          </button>
        </div>
      </div>
    </fieldset>
  </form>
</template>
