<script setup lang="ts">
import Drawer from "./components/Drawer.vue";
import DrawerItem from "./components/DrawerItem.vue";
</script>

<template>
    <Drawer logoText="OceanIAM">
        <DrawerItem tooltip="Home" to="/" />
        <DrawerItem tooltip="Applications" to="/applications" />
        <DrawerItem tooltip="Tenants" to="/tenants" />
        <DrawerItem tooltip="Users" to="/users" />

        <template #content>
            <RouterView />
        </template>
    </Drawer>
</template>
