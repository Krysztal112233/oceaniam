<script setup lang="ts">
import { Icon } from "@iconify/vue/offline";

import PageHeader from "@/components/PageHeader.vue";
import {
  usePreferencesStore,
  type ThemePreference,
} from "@/stores/preferences";
import type { SupportedLocale } from "@/i18n";

const preferences = usePreferencesStore();
const themes: Array<{ value: ThemePreference; label: string; icon: string }> = [
  {
    value: "system",
    label: "settings.system",
    icon: "material-symbols:brightness-auto-rounded",
  },
  {
    value: "light",
    label: "settings.light",
    icon: "material-symbols:light-mode-outline-rounded",
  },
  {
    value: "dark",
    label: "settings.dark",
    icon: "material-symbols:dark-mode-outline-rounded",
  },
];
const locales: Array<{ value: SupportedLocale; label: string }> = [
  { value: "zh-CN", label: "settings.chinese" },
  { value: "en-US", label: "settings.english" },
];
</script>

<template>
  <div class="page-shell max-w-5xl">
    <PageHeader
      :title="$t('settings.title')"
      :subtitle="$t('settings.subtitle')"
    />
    <div class="grid gap-6 md:grid-cols-2">
      <section class="card border border-base-300 bg-base-100 shadow-sm">
        <div class="card-body">
          <h2 class="card-title">
            <Icon icon="material-symbols:palette-outline-rounded" />
            {{ $t("settings.appearance") }}
          </h2>
          <div class="mt-3 grid grid-cols-3 gap-2">
            <button
              v-for="item in themes"
              :key="item.value"
              class="btn h-auto flex-col gap-2 py-4"
              :class="
                preferences.theme === item.value
                  ? 'btn-primary'
                  : 'btn-ghost bg-base-200'
              "
              @click="preferences.setTheme(item.value)"
            >
              <Icon :icon="item.icon" class="size-6" />
              {{ $t(item.label) }}
            </button>
          </div>
        </div>
      </section>
      <section class="card border border-base-300 bg-base-100 shadow-sm">
        <div class="card-body">
          <h2 class="card-title">
            <Icon icon="material-symbols:translate-rounded" />
            {{ $t("settings.language") }}
          </h2>
          <div class="mt-3 space-y-2">
            <label
              v-for="item in locales"
              :key="item.value"
              class="label cursor-pointer rounded-box bg-base-200 px-4 py-3"
            >
              <span>{{ $t(item.label) }}</span>
              <input
                v-model="preferences.locale"
                type="radio"
                name="locale"
                class="radio radio-primary"
                :value="item.value"
              />
            </label>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>
