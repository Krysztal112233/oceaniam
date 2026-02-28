import { createMemoryHistory, createRouter } from "vue-router";

import HomeView from "./view/HomeView.vue";
import ApplicationsView from "./view/ApplicationsView.vue";
import TenantsView from "./view/TenantsView.vue";
import UsersView from "./view/UsersView.vue";

const routes = [
    { path: "/", component: HomeView },
    { path: "/applications", component: ApplicationsView },
    { path: "/tenants", component: TenantsView },
    { path: "/users", component: UsersView },
];

export const router = createRouter({
    history: createMemoryHistory(),
    routes,
});
