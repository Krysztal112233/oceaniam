import { defineStore } from "pinia";
import { ref, watch } from "vue";

import { i18n, type SupportedLocale } from "@/i18n";

export type ThemePreference = "system" | "light" | "dark";

const THEME_KEY = "oceaniam.theme";
const LOCALE_KEY = "oceaniam.locale";

function storedTheme(): ThemePreference {
  const value = localStorage.getItem(THEME_KEY);
  return value === "light" || value === "dark" || value === "system"
    ? value
    : "system";
}

export const usePreferencesStore = defineStore("preferences", () => {
  const theme = ref<ThemePreference>(storedTheme());
  const locale = ref<SupportedLocale>(i18n.global.locale.value);
  const media = matchMedia("(prefers-color-scheme: dark)");

  function applyTheme() {
    const resolved =
      theme.value === "system"
        ? media.matches
          ? "dark"
          : "light"
        : theme.value;
    document.documentElement.dataset.theme = resolved;
    document.documentElement.style.colorScheme = resolved;
  }

  function setTheme(value: ThemePreference) {
    theme.value = value;
  }

  function setLocale(value: SupportedLocale) {
    locale.value = value;
  }

  watch(
    theme,
    (value) => {
      localStorage.setItem(THEME_KEY, value);
      applyTheme();
    },
    { immediate: true },
  );
  watch(
    locale,
    (value) => {
      localStorage.setItem(LOCALE_KEY, value);
      i18n.global.locale.value = value;
      document.documentElement.lang = value;
    },
    { immediate: true },
  );
  media.addEventListener("change", applyTheme);

  return { locale, theme, setLocale, setTheme };
});
