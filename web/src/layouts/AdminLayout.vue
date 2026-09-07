<script setup lang="ts">
import { Icon } from "@iconify/vue/offline";
import { useQueryClient } from "@tanstack/vue-query";
import { ref } from "vue";
import { useRouter } from "vue-router";

import TenantSwitcher from "@/components/TenantSwitcher.vue";
import { useAuthStore } from "@/stores/auth";

const auth = useAuthStore();
const queryClient = useQueryClient();
const router = useRouter();
const drawerOpen = ref(false);

async function signOut() {
  await auth.signout();
  queryClient.clear();
  await router.replace({ name: "login" });
}

const navigation = [
  {
    to: "/dashboard",
    label: "nav.dashboard",
    icon: "material-symbols:dashboard-outline-rounded",
  },
  {
    to: "/applications",
    label: "nav.applications",
    icon: "material-symbols:apps-rounded",
  },
  {
    to: "/secrets",
    label: "nav.secrets",
    icon: "material-symbols:key-outline-rounded",
  },
  {
    to: "/signing-keys",
    label: "nav.signingKeys",
    icon: "material-symbols:signature-rounded",
  },
  {
    to: "/audits",
    label: "nav.audits",
    icon: "material-symbols:history-rounded",
  },
  {
    to: "/settings",
    label: "nav.settings",
    icon: "material-symbols:settings-outline-rounded",
  },
] as const;
</script>

<template>
  <div class="drawer lg:drawer-open">
    <input
      id="admin-drawer"
      v-model="drawerOpen"
      type="checkbox"
      class="drawer-toggle"
    />
    <div class="drawer-content min-h-screen bg-base-200">
      <nav
        class="navbar sticky top-0 z-30 border-b border-base-300 bg-base-100/90 px-3 backdrop-blur lg:px-6"
      >
        <div class="flex-none lg:hidden">
          <label
            for="admin-drawer"
            class="btn btn-square btn-ghost"
            :aria-label="$t('common.appName')"
          >
            <Icon icon="material-symbols:menu-rounded" class="size-6" />
          </label>
        </div>
        <div class="flex-1 lg:hidden">
          <span class="text-lg font-bold">OceanIAM</span>
        </div>
        <div class="hidden flex-1 lg:block" />
        <div class="flex items-center gap-2">
          <div class="text-right max-sm:hidden">
            <div class="text-sm font-medium">
              {{ auth.profile?.name }}
            </div>
            <div class="text-base-content/55 text-xs">
              {{ auth.profile?.role || $t("common.adminConsole") }}
            </div>
          </div>
          <div class="dropdown dropdown-end">
            <button
              tabindex="0"
              class="avatar placeholder btn btn-circle btn-ghost"
              :aria-label="auth.profile?.name"
            >
              <span class="bg-neutral text-neutral-content w-9 rounded-full">{{
                auth.profile?.name?.slice(0, 1).toUpperCase()
              }}</span>
            </button>
            <ul
              tabindex="0"
              class="menu dropdown-content bg-base-100 rounded-box z-40 mt-2 w-52 border border-base-300 p-2 shadow-xl"
            >
              <li>
                <button :disabled="auth.busy" @click="signOut">
                  <Icon icon="material-symbols:logout-rounded" />
                  {{ $t("auth.signOut") }}
                </button>
              </li>
            </ul>
          </div>
        </div>
      </nav>
      <main><RouterView /></main>
    </div>

    <aside class="drawer-side z-40">
      <label
        for="admin-drawer"
        :aria-label="$t('common.closeSidebar')"
        class="drawer-overlay"
      />
      <div
        class="flex min-h-full w-72 flex-col border-r border-base-300 bg-base-100 p-3"
      >
        <RouterLink
          to="/dashboard"
          class="flex items-center gap-3 px-3 py-3"
          @click="drawerOpen = false"
        >
          <span
            class="bg-primary text-primary-content grid size-10 place-items-center rounded-xl text-lg font-black"
            >O</span
          >
          <span>
            <span class="block text-xl font-bold tracking-tight">OceanIAM</span>
            <span class="text-base-content/50 block text-xs">{{
              $t("common.adminConsole")
            }}</span>
          </span>
        </RouterLink>
        <div class="my-3">
          <TenantSwitcher />
        </div>
        <ul class="menu w-full gap-1 p-0">
          <li v-for="item in navigation" :key="item.to">
            <RouterLink
              :to="item.to"
              active-class="menu-active"
              @click="drawerOpen = false"
            >
              <Icon :icon="item.icon" class="size-5" />
              {{ $t(item.label) }}
            </RouterLink>
          </li>
        </ul>
        <div class="mt-auto px-3 py-2 text-xs text-base-content/45">
          OceanIAM Web · Vue + DaisyUI
        </div>
      </div>
    </aside>
  </div>
</template>
