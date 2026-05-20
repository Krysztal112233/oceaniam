<script setup lang="ts">
import LoginIcon from "@iconify-vue/material-symbols/login-rounded";
import SecurityIcon from "@iconify-vue/material-symbols/security-rounded";
import TenantIcon from "@iconify-vue/material-symbols/account-tree-outline-rounded";
import AuditIcon from "@iconify-vue/material-symbols/receipt-long-outline-rounded";
import AnalyticsIcon from "@iconify-vue/material-symbols/analytics-outline-rounded";
import KeyIcon from "@iconify-vue/material-symbols/key-outline-rounded";
import AppsIcon from "@iconify-vue/material-symbols/widgets-outline-rounded";
import { inject } from "vue";

const openLogin = inject<() => void>("openLogin", () => {});
</script>

<template>
    <section
        class="flex min-h-[70vh] flex-col items-center justify-center px-4"
    >
        <div class="max-w-3xl text-center">
            <h1
                class="text-5xl font-bold tracking-tight text-base-content lg:text-6xl"
            >
                OceanIAM
            </h1>
            <p
                class="mx-auto mt-4 max-w-xl text-lg leading-relaxed text-base-content/70"
            >
                Organization-RBAC 身份与访问管理平台
            </p>
            <p class="mt-2 text-sm text-base-content/50">
                基于组织角色权限模型，提供多租户应用管理、审计追踪与安全认证
            </p>

            <div class="mt-10 flex items-center justify-center gap-4">
                <button
                    type="button"
                    class="btn btn-primary btn-lg gap-2"
                    @click="openLogin"
                >
                    <LoginIcon class="h-5 w-5" />
                    登录以开始使用
                </button>
            </div>
        </div>
    </section>

    <section class="mt-8 grid gap-4 md:grid-cols-3">
        <div
            class="card border border-base-200 bg-base-100 shadow-sm transition hover:-translate-y-0.5 hover:shadow-md"
        >
            <div class="card-body gap-3">
                <div
                    class="flex h-10 w-10 items-center justify-center rounded-box bg-primary/10 text-primary"
                >
                    <SecurityIcon class="h-5 w-5" />
                </div>
                <h3 class="text-base font-semibold text-base-content">
                    ORBAC 权限模型
                </h3>
                <p class="text-sm leading-relaxed text-base-content/60">
                    基于组织-角色-活动-视图的细粒度权限控制，灵活适配复杂授权场景。
                </p>
            </div>
        </div>

        <div
            class="card border border-base-200 bg-base-100 shadow-sm transition hover:-translate-y-0.5 hover:shadow-md"
        >
            <div class="card-body gap-3">
                <div
                    class="flex h-10 w-10 items-center justify-center rounded-box bg-primary/10 text-primary"
                >
                    <TenantIcon class="h-5 w-5" />
                </div>
                <h3 class="text-base font-semibold text-base-content">
                    多租户隔离
                </h3>
                <p class="text-sm leading-relaxed text-base-content/60">
                    租户级资源隔离，每个租户拥有独立的应用、用户与配置管理空间。
                </p>
            </div>
        </div>

        <div
            class="card border border-base-200 bg-base-100 shadow-sm transition hover:-translate-y-0.5 hover:shadow-md"
        >
            <div class="card-body gap-3">
                <div
                    class="flex h-10 w-10 items-center justify-center rounded-box bg-primary/10 text-primary"
                >
                    <AuditIcon class="h-5 w-5" />
                </div>
                <h3 class="text-base font-semibold text-base-content">
                    审计日志
                </h3>
                <p class="text-sm leading-relaxed text-base-content/60">
                    完整的操作审计追踪，记录每一次关键操作，满足合规与安全审查需求。
                </p>
            </div>
        </div>

        <div
            class="card border border-base-200 bg-base-100 shadow-sm transition hover:-translate-y-0.5 hover:shadow-md"
        >
            <div class="card-body gap-3">
                <div
                    class="flex h-10 w-10 items-center justify-center rounded-box bg-secondary/10 text-secondary"
                >
                    <AnalyticsIcon class="h-5 w-5" />
                </div>
                <h3 class="text-base font-semibold text-base-content">
                    统计报表
                </h3>
                <p class="text-sm leading-relaxed text-base-content/60">
                    系统运行数据可视化，实时掌握应用活跃度、用户增长与系统资源状况。
                </p>
            </div>
        </div>

        <div
            class="card border border-base-200 bg-base-100 shadow-sm transition hover:-translate-y-0.5 hover:shadow-md"
        >
            <div class="card-body gap-3">
                <div
                    class="flex h-10 w-10 items-center justify-center rounded-box bg-secondary/10 text-secondary"
                >
                    <KeyIcon class="h-5 w-5" />
                </div>
                <h3 class="text-base font-semibold text-base-content">
                    密钥管理
                </h3>
                <p class="text-sm leading-relaxed text-base-content/60">
                    API
                    密钥与应用密钥的全生命周期管理，支持轮换、吊销与安全存储。
                </p>
            </div>
        </div>

        <div
            class="card border border-base-200 bg-base-100 shadow-sm transition hover:-translate-y-0.5 hover:shadow-md"
        >
            <div class="card-body gap-3">
                <div
                    class="flex h-10 w-10 items-center justify-center rounded-box bg-secondary/10 text-secondary"
                >
                    <AppsIcon class="h-5 w-5" />
                </div>
                <h3 class="text-base font-semibold text-base-content">
                    应用管理
                </h3>
                <p class="text-sm leading-relaxed text-base-content/60">
                    统一的第三方应用注册与配置管理，支持认证策略、令牌分发等灵活配置。
                </p>
            </div>
        </div>
    </section>
</template>
