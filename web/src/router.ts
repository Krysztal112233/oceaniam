import { createRouter, createWebHistory } from "vue-router";

import { useAuthStore } from "@/stores/auth";

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/login",
      name: "login",
      component: () => import("@/views/LoginView.vue"),
      meta: { public: true },
    },
    {
      path: "/",
      component: () => import("@/layouts/AdminLayout.vue"),
      children: [
        { path: "", redirect: "/dashboard" },
        {
          path: "dashboard",
          name: "dashboard",
          component: () => import("@/views/DashboardView.vue"),
        },
        {
          path: "applications",
          name: "applications",
          component: () => import("@/views/ApplicationsView.vue"),
        },
        {
          path: "secrets",
          name: "secrets",
          component: () => import("@/views/SecretsView.vue"),
        },
        {
          path: "signing-keys",
          name: "signing-keys",
          component: () => import("@/views/SigningKeysView.vue"),
        },
        {
          path: "audits",
          name: "audits",
          component: () => import("@/views/AuditsView.vue"),
        },
        {
          path: "settings",
          name: "settings",
          component: () => import("@/views/SettingsView.vue"),
        },
      ],
    },
    { path: "/:pathMatch(.*)*", redirect: "/dashboard" },
  ],
});

router.beforeEach(async (to) => {
  const auth = useAuthStore();
  if (!auth.ready) await auth.initialize();
  if (to.meta.public) {
    if (to.name === "login" && auth.isAuthenticated)
      return { name: "dashboard" };
    return true;
  }
  if (!auth.isAuthenticated) {
    return { name: "login", query: { redirect: to.fullPath } };
  }
  return true;
});
