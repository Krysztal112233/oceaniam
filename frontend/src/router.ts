import { createRouter, createWebHistory } from "vue-router";

import HomeView from "./view/HomeView.vue";
import OverviewView from "./view/OverviewView.vue";
import ApplicationsEntryView from "./view/ApplicationsEntryView.vue";
import ApplicationsView from "./view/ApplicationsView.vue";
import ApplicationDetailView from "./view/ApplicationDetailView.vue";
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
        component: ApplicationDetailView,
        name: "application-detail",
    },
    { path: "/users", component: UsersView, name: "users" },
    { path: "/audits", component: AuditsView, name: "audits" },
    { path: "/statistics", component: StatisticsView, name: "statistics" },
];

export const router = createRouter({
    history: createWebHistory(),
    routes,
});
