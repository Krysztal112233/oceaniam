import { createRouter, createWebHistory } from "vue-router";

import HomeView from "./view/HomeView.vue";
import OverviewView from "./view/OverviewView.vue";
import ApplicationsEntryView from "./view/ApplicationsEntryView.vue";
import ApplicationsView from "./view/ApplicationsView.vue";
import UsersEntryView from "./view/UsersEntryView.vue";
import ApiSecretsEntryView from "./view/ApiSecretsEntryView.vue";
import ApiSecretsView from "./view/ApiSecretsView.vue";
import ApiSecretDetailView from "./view/ApiSecretDetailView.vue";
import AuditsView from "./view/AuditsView.vue";
import StatisticsView from "./view/StatisticsView.vue";
import UsersView from "./view/UsersView.vue";

const routes = [
    { path: "/", component: HomeView, name: "home" },
    { path: "/overview", component: OverviewView, name: "overview" },
    {
        path: "/applications",
        component: ApplicationsEntryView,
        name: "applications-entry",
    },
    {
        path: "/tenants/:tenantId/applications",
        component: ApplicationsView,
        name: "applications",
    },
    {
        path: "/tenants/:tenantId/applications/:applicationId",
        component: () => import("./view/ApplicationDetailView.vue"),
        name: "application-detail",
    },
    {
        path: "/tenants/:tenantId/applications/:applicationId/users",
        component: UsersView,
        name: "application-users",
    },
    {
        path: "/users",
        component: UsersEntryView,
        name: "users-entry",
    },
    {
        path: "/api-secrets",
        component: ApiSecretsEntryView,
        name: "api-secrets-entry",
    },
    {
        path: "/tenants/:tenantId/api-secrets",
        component: ApiSecretsView,
        name: "api-secrets",
    },
    {
        path: "/tenants/:tenantId/api-secrets/:secretId",
        component: ApiSecretDetailView,
        name: "api-secret-detail",
    },
    { path: "/audits", component: AuditsView, name: "audits" },
    { path: "/statistics", component: StatisticsView, name: "statistics" },
];

export const router = createRouter({
    history: createWebHistory(),
    routes,
});
