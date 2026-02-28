import { createMemoryHistory, createRouter } from "vue-router";

import HomeView from "./view/HomeView.vue";
import ApplicationsView from "./view/ApplicationsView.vue";
import TenantsView from "./view/TenantsView.vue";
import UsersView from "./view/UsersView.vue";
import LoginView from "./view/LoginView.vue";

const routes = [
    { path: "/", component: HomeView },
    { path: "/applications", component: ApplicationsView },
    { path: "/tenants", component: TenantsView },
    { path: "/users", component: UsersView },
    { path: "/login", component: LoginView },
];

export const router = createRouter({
    history: createMemoryHistory(),
    routes,
});
