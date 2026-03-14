<script setup lang="ts">
import HomeIcon from "@iconify-vue/material-symbols/home-outline-rounded";
import ManageAccountsIcon from "@iconify-vue/material-symbols/manage-accounts-outline-rounded";
import MonitorHeartIcon from "@iconify-vue/material-symbols/monitor-heart-outline-rounded";
import WidgetsIcon from "@iconify-vue/material-symbols/widgets-outline-rounded";
import { ref } from "vue";
import AppLayout from "./layout/AppLayout.vue";
import DrawerGroup from "./components/DrawerGroup.vue";
import DrawerItem from "./components/DrawerItem.vue";
import AuthWidget from "./components/auth/AuthWidget.vue";
import LoginModal from "./components/auth/LoginModal.vue";
import TenantSwitcher from "./components/TenantSwitcher.vue";

const loginOpen = ref(false);
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
        <DrawerItem
            tooltip="Applications"
            to="/applications"
            :icon="WidgetsIcon"
        />
        <DrawerGroup
            label="User Management"
            :icon="ManageAccountsIcon"
            :items="[{ label: 'Users', to: '/users' }]"
        />
        <DrawerGroup
            label="Monitoring"
            :icon="MonitorHeartIcon"
            :items="[
                { label: 'Audits', to: '/audits' },
                {
                    label: 'Statistics',
                    to: '/statistics',
                },
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
