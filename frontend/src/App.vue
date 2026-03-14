<script setup lang="ts">
import { ref } from "vue";
import AppLayout from "./layout/AppLayout.vue";
import DrawerItem from "./components/DrawerItem.vue";
import AuthWidget from "./components/auth/AuthWidget.vue";
import LoginModal from "./components/auth/LoginModal.vue";
import TenantSwitcher from "./components/TenantSwitcher.vue";

const loginOpen = ref(false);
</script>

<template>
    <AppLayout brandText="OceanIAM">
        <DrawerItem tooltip="Home" to="/" />
        <DrawerItem tooltip="Applications" to="/applications" />
        <DrawerItem tooltip="Users" to="/users" />

        <template #navbar-right>
            <div class="flex items-center gap-2">
                <TenantSwitcher />
                <AuthWidget variant="navbar" @open-login="loginOpen = true" />
            </div>
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
