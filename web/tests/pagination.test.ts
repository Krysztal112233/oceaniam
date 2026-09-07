import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import PaginationBar from "@/components/PaginationBar.vue";
import { i18n } from "@/i18n";

describe("PaginationBar", () => {
  it("uses the backend's one-based page semantics", async () => {
    const wrapper = mount(PaginationBar, {
      props: { page: 1, total: 30, hasNext: true },
      global: { plugins: [i18n] },
    });
    const buttons = wrapper.findAll("button");
    expect(buttons[0]?.attributes("disabled")).toBeDefined();
    await buttons[2]?.trigger("click");
    expect(wrapper.emitted("change")).toEqual([[2]]);
  });
});
