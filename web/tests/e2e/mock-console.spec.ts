import { expect, test } from "@playwright/test";
import type { components } from "@oceaniam/sdk";

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
  const configuration: components["schemas"]["ApplicationConfigurationVO"] = {
    auth: {
      token: { issuer: "OceanIAM", audience: ["OceanIAM"] },
      password: { argon2: { m_cost: 12288, t_cost: 3, p_cost: 1 } },
    },
    registration: { enabled: false },
    development_accounts: {
      enabled: true,
      default_ttl_seconds: 3600,
      max_ttl_seconds: 86400,
    },
    oidc: { allow_insecure_loopback_redirect_uris: false },
  };
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
    if (path === "/tenants/tenant-1/applications/application-1/configuration") {
      if (method === "PATCH") {
        const patch =
          request.postDataJSON() as components["schemas"]["PatchApplicationConfigurationRequest"];
        if (patch.auth?.token?.issuer != null)
          configuration.auth.token.issuer = patch.auth.token.issuer;
        if (patch.auth?.token?.audience != null)
          configuration.auth.token.audience = patch.auth.token.audience;
        if (patch.registration?.enabled != null)
          configuration.registration.enabled = patch.registration.enabled;
        if (patch.development_accounts) {
          configuration.development_accounts = {
            enabled:
              patch.development_accounts.enabled ??
              configuration.development_accounts?.enabled ??
              true,
            default_ttl_seconds:
              patch.development_accounts.default_ttl_seconds ??
              configuration.development_accounts?.default_ttl_seconds ??
              3600,
            max_ttl_seconds:
              patch.development_accounts.max_ttl_seconds ??
              configuration.development_accounts?.max_ttl_seconds ??
              86400,
          };
        }
        if (configuration.oidc && patch.oidc)
          configuration.oidc.allow_insecure_loopback_redirect_uris =
            patch.oidc.allow_insecure_loopback_redirect_uris ??
            configuration.oidc.allow_insecure_loopback_redirect_uris;
      }
      return route.fulfill({ json: { configuration } });
    }
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
  await tenantMenu.getByRole("button", { name: /tenant-1/ }).click();
  await expect(tenantMenu).toBeHidden();

  await page.getByRole("link", { name: "Applications" }).click();
  await page.getByRole("button", { name: /application-1/ }).click();
  await expect(page.getByText("3", { exact: true })).toBeVisible();
  await expect(page.getByText("Customer portal")).toBeVisible();

  await page.getByRole("tab", { name: "Configuration" }).click();
  const configurationForm = page.getByRole("form", {
    name: "Configuration",
  });
  const saveButton = configurationForm.getByRole("button", { name: "Save" });
  await expect(
    configurationForm.getByRole("checkbox", { name: "Allow registration" }),
  ).toBeVisible();
  await expect(
    configurationForm.getByRole("checkbox", { name: "Development accounts" }),
  ).toBeVisible();
  await expect(
    configurationForm.getByRole("checkbox", {
      name: "Allow HTTP loopback redirect URIs",
    }),
  ).toBeVisible();
  await expect(saveButton).toBeDisabled();

  await configurationForm.getByLabel("Default TTL (seconds)").fill("100");
  await configurationForm.getByLabel("Maximum TTL (seconds)").fill("10");
  await saveButton.click();
  await expect(
    page.getByText("Default TTL cannot exceed maximum TTL."),
  ).toBeVisible();
  await configurationForm.getByLabel("Maximum TTL (seconds)").fill("86400");
  await configurationForm
    .getByRole("checkbox", { name: "Allow HTTP loopback redirect URIs" })
    .check();
  await expect(page.getByText("Unsaved changes")).toBeVisible();
  await saveButton.click();
  await expect(page.getByText("All changes saved")).toBeVisible();
  await expect(saveButton).toBeDisabled();

  const [configurationFormBox, saveButtonBox] = await Promise.all([
    configurationForm.boundingBox(),
    saveButton.boundingBox(),
  ]);
  expect(configurationFormBox).not.toBeNull();
  expect(saveButtonBox).not.toBeNull();
  expect(
    configurationFormBox!.x +
      configurationFormBox!.width -
      saveButtonBox!.x -
      saveButtonBox!.width,
  ).toBeLessThanOrEqual(24);

  await page.getByRole("link", { name: "Settings" }).click();
  const darkTheme = page.getByRole("button", { name: "Dark", exact: true });
  await darkTheme.click();
  await expect(darkTheme).toHaveAttribute("aria-pressed", "true");
  await expect(
    page.getByRole("button", { name: "System", exact: true }),
  ).toHaveAttribute("aria-pressed", "false");
  await expect(page.getByRole("radio", { name: "English" })).toBeChecked();

  await page.setViewportSize({ width: 390, height: 844 });
  const menuButton = page.getByRole("button", {
    name: "Open navigation menu",
  });
  await expect(menuButton).toHaveAttribute("aria-expanded", "false");
  await menuButton.click();
  await expect(
    page.getByRole("button", { name: "Close sidebar" }),
  ).toHaveAttribute("aria-expanded", "true");
  await page.keyboard.press("Escape");
  await expect(menuButton).toHaveAttribute("aria-expanded", "false");
  await expect(menuButton).toBeFocused();

  await page.getByRole("button", { name: "root" }).click();
  await page.getByRole("button", { name: "Sign out" }).click();
  await expect(page).toHaveURL(/\/login$/);
  await expect(page.getByRole("heading", { name: "Sign in" })).toBeVisible();
});
