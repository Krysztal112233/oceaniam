<script setup lang="ts">
import { toTypedSchema } from "@vee-validate/zod";
import { Icon } from "@iconify/vue/offline";
import { useForm } from "vee-validate";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";
import { z } from "zod";

import { useAuthStore } from "@/stores/auth";
import { usePreferencesStore } from "@/stores/preferences";

const auth = useAuthStore();
const preferences = usePreferencesStore();
const router = useRouter();
const route = useRoute();
const { t } = useI18n();

const schema = toTypedSchema(
  z.object({
    name: z.string().trim().min(1),
    password: z.string().min(1),
  }),
);
const { defineField, errors, handleSubmit } = useForm({
  validationSchema: schema,
});
const [name, nameAttrs] = defineField("name");
const [password, passwordAttrs] = defineField("password");

const errorMessage = computed(() => {
  if (auth.error === "invalid") return t("auth.invalid");
  if (auth.error) return t("common.requestFailed");
  return null;
});

const submit = handleSubmit(async (values) => {
  try {
    await auth.signin(values.name, values.password);
    const requested =
      typeof route.query.redirect === "string"
        ? route.query.redirect
        : "/dashboard";
    const safeRedirect =
      requested.startsWith("/") && !requested.startsWith("//")
        ? requested
        : "/dashboard";
    await router.replace(safeRedirect);
  } catch {
    // The auth store exposes a localized error state.
  }
});
</script>

<template>
  <main class="hero min-h-screen bg-base-200 px-4 py-12">
    <div
      class="hero-content w-full max-w-5xl flex-col gap-10 lg:flex-row lg:gap-20"
    >
      <section class="max-w-xl text-center lg:text-left">
        <div class="mb-5 inline-flex items-center gap-3">
          <span
            class="bg-primary text-primary-content grid size-14 place-items-center rounded-2xl text-2xl font-black shadow-lg"
            >O</span
          >
          <span class="text-3xl font-bold tracking-tight">OceanIAM</span>
        </div>
        <h1 class="text-4xl font-bold sm:text-5xl">
          {{ $t("auth.welcome") }}
        </h1>
        <p class="text-base-content/65 mt-5 text-lg">
          {{ $t("auth.description") }}
        </p>
        <div class="mt-8 flex justify-center gap-2 lg:justify-start">
          <select
            v-model="preferences.locale"
            class="select select-sm w-auto"
            :aria-label="$t('settings.language')"
          >
            <option value="zh-CN">简体中文</option>
            <option value="en-US">English</option>
          </select>
          <select
            v-model="preferences.theme"
            class="select select-sm w-auto"
            :aria-label="$t('settings.theme')"
          >
            <option value="system">
              {{ $t("settings.system") }}
            </option>
            <option value="light">
              {{ $t("settings.light") }}
            </option>
            <option value="dark">
              {{ $t("settings.dark") }}
            </option>
          </select>
        </div>
      </section>

      <section
        class="card w-full max-w-md border border-base-300 bg-base-100 shadow-xl"
      >
        <div class="card-body p-6 sm:p-8">
          <h2 class="card-title text-2xl">
            {{ $t("auth.signIn") }}
          </h2>
          <form class="mt-3 space-y-4" @submit="submit">
            <label class="fieldset">
              <span class="fieldset-legend">{{ $t("auth.name") }}</span>
              <label
                class="input w-full"
                :class="{ 'input-error': errors.name }"
              >
                <Icon
                  icon="material-symbols:person-outline-rounded"
                  class="size-5 opacity-50"
                />
                <input
                  v-model="name"
                  v-bind="nameAttrs"
                  name="name"
                  autocomplete="username"
                  autofocus
                />
              </label>
            </label>
            <label class="fieldset">
              <span class="fieldset-legend">{{ $t("auth.password") }}</span>
              <label
                class="input w-full"
                :class="{ 'input-error': errors.password }"
              >
                <Icon
                  icon="material-symbols:lock-outline-rounded"
                  class="size-5 opacity-50"
                />
                <input
                  v-model="password"
                  v-bind="passwordAttrs"
                  name="password"
                  type="password"
                  autocomplete="current-password"
                />
              </label>
            </label>
            <div
              v-if="errorMessage"
              role="alert"
              class="alert alert-error alert-soft text-sm"
            >
              <Icon
                icon="material-symbols:error-outline-rounded"
                class="size-5"
              />
              <span>{{ errorMessage }}</span>
            </div>
            <button
              class="btn btn-primary mt-2 w-full"
              :disabled="auth.busy"
              type="submit"
            >
              <span
                v-if="auth.busy"
                class="loading loading-spinner loading-sm"
              />
              {{ $t("auth.signIn") }}
            </button>
          </form>
        </div>
      </section>
    </div>
  </main>
</template>
