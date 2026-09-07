import { expect, test } from "@playwright/test";

function fakeJwt() {
  const payload = Buffer.from(
    JSON.stringify({ exp: Math.floor(Date.now() / 1000) + 3600 }),
  ).toString("base64url");
  return `header.${payload}.signature`;
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() =>
    localStorage.setItem("oceaniam.locale", "en-US"),
  );
  await page.route("**/api/**", async (route) => {
    const request = route.request();
    const path = new URL(request.url()).pathname.replace(/^\/api/, "");
    const method = request.method();
    if (path === "/auth/tokens" && method === "POST")
      return route.fulfill({ json: { jwt: fakeJwt() } });
    if (path === "/administrators/me")
      return route.fulfill({
        json: { id: "admin-1", name: "root", role: "Root", permissions: [] },
      });
    if (path === "/tenants")
      return route.fulfill({
        json: {
          items: [{ id: "tenant-1", comment: "Primary tenant" }],
          page_info: { total: 1, has_next: false },
        },
      });
    if (path === "/statistics")
      return route.fulfill({
        json: {
          total_tenants: 1,
          total_applications: 1,
          total_administrators: 1,
          total_application_users: 3,
          total_active_secrets: 0,
        },
      });
    if (path === "/statistics/trends")
      return route.fulfill({
        json: {
          granularity: "day",
          range: 30,
          tenants: [],
          applications: [],
          users: [],
          administrators: [],
        },
      });
    if (path === "/tenants/tenant-1/applications")
      return route.fulfill({
        json: {
          items: [
            {
              id: "application-1",
              tenant_id: "tenant-1",
              comment: "Customer portal",
            },
          ],
          page_info: { total: 1, has_next: false },
        },
      });
    if (path === "/tenants/tenant-1/applications/application-1/statistics")
      return route.fulfill({ json: { total_users: 3, total_active_keys: 1 } });
    if (path === "/tenants/tenant-1/applications/application-1/configuration")
      return route.fulfill({
        json: {
          configuration: {
            auth: {
              token: { issuer: "OceanIAM", audience: ["OceanIAM"] },
              password: {
                argon2: { m_cost: 12288, t_cost: 3, p_cost: 1 },
              },
            },
            registration: { enabled: false },
          },
        },
      });
    return route.fulfill({ json: {} });
  });
});

test("signs in and expands an application card", async ({ page }) => {
  await page.goto("/login");
  await page.getByLabel("Administrator name").fill("root");
  await page.getByLabel("Password").fill("a secure password");
  await page.getByRole("button", { name: "Sign in" }).click();
  await expect(
    page.getByRole("heading", { name: "Platform overview" }),
  ).toBeVisible();

  await page.getByRole("button", { name: /Current tenant/ }).click();
  const sidebar = page.locator(".drawer-side > div");
  const tenantMenu = page.locator(".drawer-side .dropdown-content");
  await expect(tenantMenu).toBeVisible();
  const [sidebarBox, tenantMenuBox] = await Promise.all([
    sidebar.boundingBox(),
    tenantMenu.boundingBox(),
  ]);
  expect(sidebarBox).not.toBeNull();
  expect(tenantMenuBox).not.toBeNull();
  expect(tenantMenuBox!.x).toBeGreaterThanOrEqual(sidebarBox!.x);
  expect(tenantMenuBox!.x + tenantMenuBox!.width).toBeLessThanOrEqual(
    sidebarBox!.x + sidebarBox!.width,
  );
  await page.getByRole("heading", { name: "Platform overview" }).click();
  await expect(tenantMenu).toBeHidden();

  await page.getByRole("link", { name: "Applications" }).click();
  await page.getByRole("button", { name: /application-1/ }).click();
  await expect(page.getByText("3", { exact: true })).toBeVisible();
  await expect(page.getByText("Customer portal")).toBeVisible();

  await page.getByRole("tab", { name: "Configuration" }).click();
  const tokenPanel = page.locator("section").filter({
    has: page.getByRole("heading", { name: "Token", exact: true }),
  });
  const saveButton = tokenPanel.getByRole("button", { name: "Save" });
  await expect(
    tokenPanel.getByRole("checkbox", { name: "Allow registration" }),
  ).toBeVisible();
  await expect(saveButton).toBeVisible();
  const [tokenPanelBox, saveButtonBox] = await Promise.all([
    tokenPanel.boundingBox(),
    saveButton.boundingBox(),
  ]);
  expect(tokenPanelBox).not.toBeNull();
  expect(saveButtonBox).not.toBeNull();
  expect(
    tokenPanelBox!.x +
      tokenPanelBox!.width -
      saveButtonBox!.x -
      saveButtonBox!.width,
  ).toBeLessThanOrEqual(24);

  await page.getByRole("button", { name: "root" }).click();
  await page.getByRole("button", { name: "Sign out" }).click();
  await expect(page).toHaveURL(/\/login$/);
  await expect(page.getByRole("heading", { name: "Sign in" })).toBeVisible();
});
