import { createMemoryHistory, createRouter } from "vue-router";

import HomeView from "./view/HomeView.vue";
import ApplicationsEntryView from "./view/ApplicationsEntryView.vue";
import ApplicationsView from "./view/ApplicationsView.vue";
import AuditsView from "./view/AuditsView.vue";
import StatisticsView from "./view/StatisticsView.vue";
import UsersView from "./view/UsersView.vue";

const routes = [
    { path: "/", component: HomeView, name: "home" },
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
    { path: "/users", component: UsersView, name: "users" },
    { path: "/audits", component: AuditsView, name: "audits" },
    { path: "/statistics", component: StatisticsView, name: "statistics" },
];

export const router = createRouter({
    history: createMemoryHistory(),
    routes,
});
