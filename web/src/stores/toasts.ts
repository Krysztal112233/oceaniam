import { defineStore } from "pinia";
import { ref } from "vue";

export type ToastKind = "success" | "error" | "info";
export interface ToastMessage {
  id: number;
  kind: ToastKind;
  message: string;
}

export const useToastsStore = defineStore("toasts", () => {
  const messages = ref<ToastMessage[]>([]);
  let nextId = 1;

  function show(message: string, kind: ToastKind = "info") {
    const id = nextId++;
    messages.value.push({ id, kind, message });
    window.setTimeout(() => dismiss(id), 4500);
  }

  function dismiss(id: number) {
    messages.value = messages.value.filter((item) => item.id !== id);
  }

  return { messages, dismiss, show };
});
