<script setup lang="ts">
import { useAuthStore } from "../stores/auth";
import DashboardView from "./DashboardView.vue";
import LandingView from "./LandingView.vue";

const authStore = useAuthStore();
</script>

<template>
    <LandingView v-if="!authStore.isLoggedIn" />
    <DashboardView v-else />
</template>
