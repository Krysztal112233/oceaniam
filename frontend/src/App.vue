<script setup lang="ts">
import HomeIcon from "@iconify-vue/material-symbols/home-outline-rounded";
import WidgetsIcon from "@iconify-vue/material-symbols/widgets-outline-rounded";
import SettingsIcon from "@iconify-vue/material-symbols/settings-outline-rounded";
import KeyIcon from "@iconify-vue/material-symbols/key-outline-rounded";
import GroupIcon from "@iconify-vue/material-symbols/group-outline-rounded";
import GridViewIcon from "@iconify-vue/material-symbols/grid-view-outline-rounded";
import MonitoringIcon from "@iconify-vue/material-symbols/monitoring-rounded";
import DashboardIcon from "@iconify-vue/material-symbols/dashboard-outline-rounded";
import ReceiptLongIcon from "@iconify-vue/material-symbols/receipt-long-outline-rounded";
import BarChartIcon from "@iconify-vue/material-symbols/bar-chart-rounded";
import { ref, provide } from "vue";
import AppLayout from "./layout/AppLayout.vue";
import DrawerGroup from "./components/DrawerGroup.vue";
import DrawerItem from "./components/DrawerItem.vue";
import AuthWidget from "./components/auth/AuthWidget.vue";
import LoginModal from "./components/auth/LoginModal.vue";
import TenantSwitcher from "./components/TenantSwitcher.vue";

const loginOpen = ref(false);
provide("openLogin", () => {
    loginOpen.value = true;
});
</script>

<template>
    <AppLayout brandText="OceanIAM">
        <template #navbar-brand-extra>
            <div class="flex items-center gap-3">
                <div class="h-6 w-px bg-base-300"></div>
                <TenantSwitcher />
            </div>
        </template>

        <DrawerItem tooltip="Home" to="/" :icon="HomeIcon" />
        <DrawerGroup
            label="Applications"
            :icon="WidgetsIcon"
            :items="[
                {
                    label: 'Application',
                    icon: GridViewIcon,
                    to: '/applications',
                },
                { label: 'Users', icon: GroupIcon, to: '/users' },
            ]"
        />
        <DrawerGroup
            label="Platform"
            :icon="SettingsIcon"
            :items="[
                { label: 'API Secrets', icon: KeyIcon, to: '/api-secrets' },
            ]"
        />
        <DrawerGroup
            label="Observability"
            :icon="MonitoringIcon"
            :items="[
                { label: 'Overview', icon: DashboardIcon, to: '/overview' },
                { label: 'Audits', icon: ReceiptLongIcon, to: '/audits' },
                { label: 'Statistics', icon: BarChartIcon, to: '/statistics' },
            ]"
        />

        <template #navbar-right>
            <AuthWidget variant="navbar" @open-login="loginOpen = true" />
        </template>

        <template #content>
            <RouterView />
        </template>
    </AppLayout>

    <LoginModal
        :open="loginOpen"
        @close="loginOpen = false"
        @success="loginOpen = false"
    />
</template>
