<script setup lang="ts">
import { Icon } from "@iconify/vue/offline";
import {
  keepPreviousData,
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/vue-query";
import type { components, operations } from "@oceaniam/sdk";
import { unwrap } from "@oceaniam/sdk";
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { z } from "zod";

import ConfirmDialog from "@/components/ConfirmDialog.vue";
import EmptyState from "@/components/EmptyState.vue";
import ErrorState from "@/components/ErrorState.vue";
import ModalDialog from "@/components/ModalDialog.vue";
import PaginationBar from "@/components/PaginationBar.vue";
import {
  developmentOptions,
  type AccountType,
  type TtlPreset,
} from "@/features/users/development";
import { api, APPLICATION_AUTH_HEADER } from "@/services/api";
import { useToastsStore } from "@/stores/toasts";

type User = components["schemas"]["ApplicationUserVO"];
type CreateUserRequest = components["schemas"]["CreateApplicationUserRequest"];
type SearchQuery = NonNullable<
  operations["search_application_users"]["parameters"]["query"]
>;
type SearchField = "nickname" | "email" | "phone" | "id";

const props = defineProps<{ tenantId: string; applicationId: string }>();
const { t, locale } = useI18n();
const toasts = useToastsStore();
const queryClient = useQueryClient();
const page = ref(1);
const searchField = ref<SearchField>("nickname");
const searchInput = ref("");
const appliedSearch = ref("");
const createOpen = ref(false);
const createError = ref<string | null>(null);
const createdExpiry = ref<string | null>(null);
const nickname = ref("");
const contactKind = ref<"email" | "phone">("email");
const contact = ref("");
const password = ref("");
const accountType = ref<AccountType>("permanent");
const ttlPreset = ref<TtlPreset>("3600");
const customTtl = ref(3600);
const editUser = ref<User | null>(null);
const editNickname = ref("");
const passwordUser = ref<User | null>(null);
const newPassword = ref("");
const deleteUser = ref<User | null>(null);

const queryKey = computed(() => [
  "tenant",
  props.tenantId,
  "application",
  props.applicationId,
  "users",
  page.value,
  searchField.value,
  appliedSearch.value,
]);

function searchQuery(): SearchQuery {
  const base: SearchQuery = {
    page: page.value,
    per_page: 20,
    sort_order: "desc",
  };
  const value = appliedSearch.value.trim();
  if (!value) return base;
  if (searchField.value === "id") base.by_id = value;
  else if (searchField.value === "email") base.by_email = value;
  else if (searchField.value === "phone") base.by_phone = value;
  else base.by_nickname = value;
  return base;
}

const usersQuery = useQuery({
  queryKey,
  placeholderData: keepPreviousData,
  queryFn: async () => {
    const params = {
      header: APPLICATION_AUTH_HEADER,
      path: { tenant_id: props.tenantId, application_id: props.applicationId },
    };
    if (!appliedSearch.value) {
      return unwrap(
        await api.GET(
          "/tenants/{tenant_id}/applications/{application_id}/users",
          {
            params: {
              ...params,
              query: { page: page.value, per_page: 20, sort_order: "desc" },
            },
          },
        ),
      );
    }
    return unwrap(
      await api.GET(
        "/tenants/{tenant_id}/applications/{application_id}/users/search",
        {
          params: { ...params, query: searchQuery() },
        },
      ),
    );
  },
});

function applySearch() {
  page.value = 1;
  appliedSearch.value = searchInput.value.trim();
}

function resetSearch() {
  searchInput.value = "";
  appliedSearch.value = "";
  page.value = 1;
}

function resetCreateForm() {
  nickname.value = "";
  contactKind.value = "email";
  contact.value = "";
  password.value = "";
  accountType.value = "permanent";
  ttlPreset.value = "3600";
  customTtl.value = 3600;
  createError.value = null;
}

function createPayload(): CreateUserRequest | null {
  const schema = z.object({
    nickname: z
      .string()
      .trim()
      .refine((value) => !value || value.length >= 4),
    contact:
      contactKind.value === "email"
        ? z.string().email()
        : z.string().trim().min(4),
    password: z.string().min(12),
  });
  const parsed = schema.safeParse({
    nickname: nickname.value,
    contact: contact.value,
    password: password.value,
  });
  if (!parsed.success) {
    createError.value =
      parsed.error.issues[0]?.message ?? t("common.requestFailed");
    return null;
  }

  const payload: CreateUserRequest = { password: parsed.data.password };
  if (parsed.data.nickname) payload.nickname = parsed.data.nickname;
  if (contactKind.value === "email") payload.email = parsed.data.contact;
  else payload.phone = parsed.data.contact;
  try {
    const development = developmentOptions(
      accountType.value,
      ttlPreset.value,
      customTtl.value,
    );
    if (development) payload.development = development;
  } catch (error) {
    createError.value =
      error instanceof RangeError
        ? t("users.ttlHint")
        : error instanceof Error
          ? error.message
          : t("common.requestFailed");
    return null;
  }
  createError.value = null;
  return payload;
}

async function invalidateUsers() {
  await queryClient.invalidateQueries({
    queryKey: [
      "tenant",
      props.tenantId,
      "application",
      props.applicationId,
      "users",
    ],
  });
  await queryClient.invalidateQueries({
    queryKey: [
      "tenant",
      props.tenantId,
      "application",
      props.applicationId,
      "statistics",
    ],
  });
}

const createMutation = useMutation({
  mutationFn: async () => {
    const body = createPayload();
    if (!body) throw new Error(createError.value ?? "Invalid form");
    return unwrap(
      await api.POST(
        "/tenants/{tenant_id}/applications/{application_id}/users",
        {
          params: {
            header: APPLICATION_AUTH_HEADER,
            path: {
              tenant_id: props.tenantId,
              application_id: props.applicationId,
            },
          },
          body,
        },
      ),
    );
  },
  onSuccess: async (created) => {
    createdExpiry.value = created.expires_at ?? null;
    await invalidateUsers();
    if (!created.expires_at) {
      createOpen.value = false;
      resetCreateForm();
    }
    toasts.show(t("users.create"), "success");
  },
  onError: (error) => {
    if (!createError.value)
      createError.value =
        error instanceof Error ? error.message : t("common.requestFailed");
  },
});

const nicknameMutation = useMutation({
  mutationFn: async () => {
    if (!editUser.value) throw new Error("No user selected");
    return unwrap(
      await api.PATCH(
        "/tenants/{tenant_id}/applications/{application_id}/users/{user_id}",
        {
          params: {
            header: APPLICATION_AUTH_HEADER,
            path: {
              tenant_id: props.tenantId,
              application_id: props.applicationId,
              user_id: editUser.value.id,
            },
          },
          body: { nickname: editNickname.value.trim() },
        },
      ),
    );
  },
  onSuccess: async () => {
    editUser.value = null;
    await invalidateUsers();
    toasts.show(t("common.saved"), "success");
  },
  onError: (error) =>
    toasts.show(
      error instanceof Error ? error.message : t("common.requestFailed"),
      "error",
    ),
});

const passwordMutation = useMutation({
  mutationFn: async () => {
    if (!passwordUser.value) throw new Error("No user selected");
    if (newPassword.value.length < 12) throw new Error(t("users.passwordHint"));
    return unwrap(
      await api.PATCH(
        "/tenants/{tenant_id}/applications/{application_id}/users/{user_id}/credentials",
        {
          params: {
            header: APPLICATION_AUTH_HEADER,
            path: {
              tenant_id: props.tenantId,
              application_id: props.applicationId,
              user_id: passwordUser.value.id,
            },
          },
          body: { password: newPassword.value },
        },
      ),
    );
  },
  onSuccess: () => {
    passwordUser.value = null;
    newPassword.value = "";
    toasts.show(t("common.saved"), "success");
  },
  onError: (error) =>
    toasts.show(
      error instanceof Error ? error.message : t("common.requestFailed"),
      "error",
    ),
});

const deleteMutation = useMutation({
  mutationFn: async () => {
    if (!deleteUser.value) throw new Error("No user selected");
    return unwrap(
      await api.DELETE(
        "/tenants/{tenant_id}/applications/{application_id}/users/{user_id}",
        {
          params: {
            header: APPLICATION_AUTH_HEADER,
            path: {
              tenant_id: props.tenantId,
              application_id: props.applicationId,
              user_id: deleteUser.value.id,
            },
          },
        },
      ),
    );
  },
  onSuccess: async () => {
    deleteUser.value = null;
    await invalidateUsers();
    toasts.show(t("common.deleted"), "success");
  },
  onError: (error) =>
    toasts.show(
      error instanceof Error ? error.message : t("common.requestFailed"),
      "error",
    ),
});

function openNickname(user: User) {
  editUser.value = user;
  editNickname.value = user.nickname;
}

function closeCreate() {
  createOpen.value = false;
  createdExpiry.value = null;
  resetCreateForm();
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat(locale.value, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}
</script>

<template>
  <div class="space-y-4">
    <div
      class="flex flex-col gap-3 md:flex-row md:items-end md:justify-between"
    >
      <form
        class="flex min-w-0 flex-1 flex-col gap-2 sm:flex-row"
        @submit.prevent="applySearch"
      >
        <select v-model="searchField" class="select sm:w-36">
          <option value="nickname">
            {{ $t("users.nickname") }}
          </option>
          <option value="email">
            {{ $t("users.email") }}
          </option>
          <option value="phone">
            {{ $t("users.phone") }}
          </option>
          <option value="id">ID</option>
        </select>
        <label class="input min-w-0 flex-1">
          <Icon
            icon="material-symbols:search-rounded"
            class="size-5 opacity-50"
          />
          <input v-model="searchInput" :placeholder="$t('users.searchHint')" />
        </label>
        <button class="btn" type="submit">
          {{ $t("common.search") }}
        </button>
        <button
          v-if="appliedSearch"
          class="btn btn-ghost"
          type="button"
          @click="resetSearch"
        >
          {{ $t("common.reset") }}
        </button>
      </form>
      <button class="btn btn-primary" @click="createOpen = true">
        <Icon icon="material-symbols:person-add-outline-rounded" />
        {{ $t("users.create") }}
      </button>
    </div>

    <ErrorState
      v-if="usersQuery.isError.value"
      :message="usersQuery.error.value?.message"
      @retry="usersQuery.refetch()"
    />
    <div v-else-if="usersQuery.isPending.value" class="space-y-2">
      <div v-for="index in 4" :key="index" class="skeleton h-16 w-full" />
    </div>
    <EmptyState
      v-else-if="!usersQuery.data.value?.items.length"
      icon="material-symbols:group-off-outline-rounded"
      :title="$t('users.empty')"
    />
    <div v-else class="overflow-x-auto rounded-box border border-base-300">
      <table class="table bg-base-100">
        <thead>
          <tr>
            <th>{{ $t("users.nickname") }}</th>
            <th>{{ $t("users.email") }} / {{ $t("users.phone") }}</th>
            <th>ID</th>
            <th class="text-right">
              {{ $t("common.actions") }}
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="user in usersQuery.data.value.items" :key="user.id">
            <td class="font-medium">
              {{ user.nickname }}
            </td>
            <td>
              <span class="block">{{ user.email || user.phone || "—" }}</span>
            </td>
            <td>
              <code class="text-xs">{{ user.id }}</code>
            </td>
            <td>
              <div class="flex justify-end gap-1">
                <button
                  class="btn btn-ghost btn-sm btn-square"
                  :aria-label="$t('users.changeNickname')"
                  @click="openNickname(user)"
                >
                  <Icon icon="material-symbols:edit-outline-rounded" />
                </button>
                <button
                  class="btn btn-ghost btn-sm btn-square"
                  :aria-label="$t('users.changePassword')"
                  @click="
                    passwordUser = user;
                    newPassword = '';
                  "
                >
                  <Icon icon="material-symbols:password-rounded" />
                </button>
                <button
                  class="btn btn-ghost btn-sm btn-square text-error"
                  :aria-label="$t('common.delete')"
                  @click="deleteUser = user"
                >
                  <Icon icon="material-symbols:delete-outline-rounded" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <PaginationBar
      v-if="usersQuery.data.value"
      :page="page"
      :total="usersQuery.data.value.page_info.total"
      :has-next="usersQuery.data.value.page_info.has_next"
      @change="page = $event"
    />
  </div>

  <ModalDialog
    :open="createOpen"
    :title="$t('users.create')"
    @close="closeCreate"
  >
    <div v-if="createdExpiry" class="alert alert-success alert-soft">
      <Icon icon="material-symbols:check-circle-outline-rounded" />
      <span>{{ $t("users.expiresAt") }}: {{ formatDate(createdExpiry) }}</span>
      <button class="btn btn-sm" @click="closeCreate">
        {{ $t("common.close") }}
      </button>
    </div>
    <form v-else class="space-y-3" @submit.prevent="createMutation.mutate()">
      <label class="fieldset"
        ><span class="fieldset-legend"
          >{{ $t("users.nickname") }} ({{ $t("common.optional") }})</span
        ><input v-model="nickname" class="input w-full" minlength="4"
      /></label>
      <div class="join w-full">
        <select v-model="contactKind" class="select join-item w-32">
          <option value="email">
            {{ $t("users.email") }}
          </option>
          <option value="phone">
            {{ $t("users.phone") }}
          </option>
        </select>
        <input
          v-model="contact"
          class="input join-item min-w-0 flex-1"
          :type="contactKind === 'email' ? 'email' : 'tel'"
          required
        />
      </div>
      <label class="fieldset"
        ><span class="fieldset-legend">{{ $t("users.password") }}</span
        ><input
          v-model="password"
          class="input w-full"
          type="password"
          minlength="12"
          required
          autocomplete="new-password"
        /><span class="label">{{ $t("users.passwordHint") }}</span></label
      >
      <fieldset class="fieldset">
        <legend class="fieldset-legend">
          {{ $t("users.accountType") }}
        </legend>
        <div class="join w-full">
          <button
            type="button"
            class="btn join-item flex-1"
            :class="{ 'btn-primary': accountType === 'permanent' }"
            @click="accountType = 'permanent'"
          >
            {{ $t("users.permanent") }}
          </button>
          <button
            type="button"
            class="btn join-item flex-1"
            :class="{ 'btn-primary': accountType === 'development' }"
            @click="accountType = 'development'"
          >
            {{ $t("users.development") }}
          </button>
        </div>
      </fieldset>
      <div
        v-if="accountType === 'development'"
        class="rounded-box bg-base-200 p-3"
      >
        <label class="fieldset"
          ><span class="fieldset-legend">{{ $t("users.ttl") }}</span
          ><select v-model="ttlPreset" class="select w-full">
            <option value="3600">{{ $t("users.oneHour") }}</option>
            <option value="86400">{{ $t("users.oneDay") }}</option>
            <option value="604800">{{ $t("users.sevenDays") }}</option>
            <option value="custom">{{ $t("users.customTtl") }}</option>
          </select></label
        >
        <label v-if="ttlPreset === 'custom'" class="fieldset"
          ><span class="fieldset-legend">{{ $t("users.customTtl") }}</span
          ><input
            v-model.number="customTtl"
            class="input w-full"
            type="number"
            min="1"
            max="2147483647"
            required
        /></label>
      </div>
      <div v-if="createError" class="alert alert-error alert-soft py-2 text-sm">
        {{ createError }}
      </div>
      <div class="modal-action">
        <button
          type="button"
          class="btn"
          :disabled="createMutation.isPending.value"
          @click="closeCreate"
        >
          {{ $t("common.cancel") }}</button
        ><button
          class="btn btn-primary"
          :disabled="createMutation.isPending.value"
        >
          <span
            v-if="createMutation.isPending.value"
            class="loading loading-spinner loading-sm"
          />{{ $t("common.create") }}
        </button>
      </div>
    </form>
  </ModalDialog>

  <ModalDialog
    :open="Boolean(editUser)"
    :title="$t('users.changeNickname')"
    @close="editUser = null"
  >
    <form @submit.prevent="nicknameMutation.mutate()">
      <input
        v-model="editNickname"
        class="input w-full"
        minlength="4"
        required
      />
      <div class="modal-action">
        <button type="button" class="btn" @click="editUser = null">
          {{ $t("common.cancel") }}</button
        ><button
          class="btn btn-primary"
          :disabled="nicknameMutation.isPending.value"
        >
          {{ $t("common.save") }}
        </button>
      </div>
    </form>
  </ModalDialog>
  <ModalDialog
    :open="Boolean(passwordUser)"
    :title="$t('users.changePassword')"
    @close="passwordUser = null"
  >
    <form @submit.prevent="passwordMutation.mutate()">
      <input
        v-model="newPassword"
        class="input w-full"
        type="password"
        minlength="12"
        required
        autocomplete="new-password"
      /><span class="label">{{ $t("users.passwordHint") }}</span>
      <div class="modal-action">
        <button type="button" class="btn" @click="passwordUser = null">
          {{ $t("common.cancel") }}</button
        ><button
          class="btn btn-primary"
          :disabled="passwordMutation.isPending.value"
        >
          {{ $t("common.save") }}
        </button>
      </div>
    </form>
  </ModalDialog>
  <ConfirmDialog
    :open="Boolean(deleteUser)"
    :title="$t('users.deleteTitle')"
    :busy="deleteMutation.isPending.value"
    @close="deleteUser = null"
    @confirm="deleteMutation.mutate()"
  />
</template>
