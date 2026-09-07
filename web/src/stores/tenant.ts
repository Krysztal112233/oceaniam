import { defineStore } from "pinia";
import { ref } from "vue";

const TENANT_KEY = "oceaniam.tenant";

export const useTenantStore = defineStore("tenant", () => {
  const currentId = ref<string | null>(localStorage.getItem(TENANT_KEY));
  const revision = ref(0);

  function select(id: string | null) {
    if (currentId.value === id) return;
    currentId.value = id;
    revision.value += 1;
    if (id) localStorage.setItem(TENANT_KEY, id);
    else localStorage.removeItem(TENANT_KEY);
  }

  return { currentId, revision, select };
});
