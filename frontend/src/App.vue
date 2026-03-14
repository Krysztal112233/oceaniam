<script setup lang="ts">
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

        <DrawerItem tooltip="Home" to="/" />
        <DrawerItem tooltip="Applications" to="/applications" />
        <DrawerGroup
            label="User Management"
            :items="[{ label: 'Users', to: '/users' }]"
        />
        <DrawerGroup
            label="Monitoring"
            :items="[
                { label: 'Audits', to: '/audits' },
                { label: 'Statistics', to: '/statistics' },
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
