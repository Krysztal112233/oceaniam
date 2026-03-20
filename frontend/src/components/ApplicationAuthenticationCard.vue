<script setup lang="ts">
import type { ApplicationConfigurationVO } from "../../packages/sdk/src/types/ApplicationConfigurationVO";
import ConfigurationCard from "./config/ConfigurationCard.vue";
import ConfigurationItemList from "./config/ConfigurationItemList.vue";
import ConfigurationItemText from "./config/ConfigurationItemText.vue";

defineProps<{
    configuration: ApplicationConfigurationVO | null;
}>();
</script>

<template>
    <ConfigurationCard
        v-if="configuration"
        title="Authentication Configuration"
        description="查看当前 Application 的认证发行方与受众相关配置。"
    >
        <ConfigurationItemText
            title="Issuer"
            :value="configuration.authentication.issuer"
        />

        <ConfigurationItemList
            title="Audience"
            :empty="configuration.authentication.audience.length === 0"
            empty-text="暂无 audience 配置"
        >
            <template
                v-for="audience in configuration.authentication.audience"
                :key="audience"
            >
                <ConfigurationItemText :value="audience" />
            </template>
        </ConfigurationItemList>
    </ConfigurationCard>
</template>
