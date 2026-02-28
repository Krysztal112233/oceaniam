<script setup lang="ts">
import { ref } from "vue";
import Drawer from "./components/Drawer.vue";
import DrawerItem from "./components/DrawerItem.vue";
import AuthWidget from "./components/auth/AuthWidget.vue";
import LoginModal from "./components/auth/LoginModal.vue";

const loginOpen = ref(false);
</script>

<template>
    <Drawer logoText="OceanIAM">
        <DrawerItem tooltip="Home" to="/" />
        <DrawerItem tooltip="Applications" to="/applications" />
        <DrawerItem tooltip="Tenants" to="/tenants" />
        <DrawerItem tooltip="Users" to="/users" />

        <template #footer>
            <AuthWidget @open-login="loginOpen = true" />
        </template>

        <template #content>
            <RouterView />
        </template>
    </Drawer>

    <LoginModal
        :open="loginOpen"
        @close="loginOpen = false"
        @success="loginOpen = false"
    />
</template>
