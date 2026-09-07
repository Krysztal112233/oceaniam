import pluginVue from "eslint-plugin-vue";
import {
  defineConfigWithVueTs,
  vueTsConfigs,
} from "@vue/eslint-config-typescript";
import eslintConfigPrettier from "eslint-config-prettier";

export default defineConfigWithVueTs(
  {
    ignores: [
      "dist/**",
      "coverage/**",
      "playwright-report/**",
      "test-results/**",
    ],
  },
  pluginVue.configs["flat/recommended"],
  vueTsConfigs.recommendedTypeChecked,
  {
    files: ["src/**/*.{ts,vue}", "tests/**/*.ts"],
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      "vue/multi-word-component-names": "off",
      "vue/no-v-html": "off",
    },
  },
  eslintConfigPrettier,
);
