import { expect, test } from "@playwright/test";

test.describe("real backend smoke", () => {
  test.skip(
    !process.env.OCEANIAM_E2E_ROOT_PASSWORD,
    "Set OCEANIAM_E2E_ROOT_PASSWORD for the isolated backend.",
  );

  test("authenticates and persists application configuration", async ({
    page,
  }) => {
    const suffix = Date.now().toString(36);
    await page.addInitScript(() =>
      localStorage.setItem("oceaniam.locale", "en-US"),
    );
    await page.goto("/login");
    await page.getByLabel("Administrator name").fill("root");
    await page
      .getByLabel("Password")
      .fill(process.env.OCEANIAM_E2E_ROOT_PASSWORD!);
    await page.getByRole("button", { name: "Sign in" }).click();
    await expect(
      page.getByRole("heading", { name: "Platform overview" }),
    ).toBeVisible();
    await page.getByText("Current tenant").click();
    await page.getByRole("button", { name: "Create tenant" }).click();
    await page
      .getByPlaceholder("Optional description for this tenant")
      .fill(`Playwright ${suffix}`);
    await page
      .getByRole("dialog")
      .getByRole("button", { name: "Create", exact: true })
      .click();
    await expect(page.getByRole("dialog")).toBeHidden();
    await page.getByText("Current tenant").click();
    await expect(page.getByText(`Playwright ${suffix}`)).toBeVisible();
    await page.getByRole("heading", { name: "Platform overview" }).click();

    await page.getByRole("link", { name: "Applications" }).click();
    await page
      .getByRole("button", { name: "Create application", exact: true })
      .first()
      .click();
    await page
      .getByRole("dialog")
      .locator("textarea")
      .fill(`Playwright application ${suffix}`);
    await page
      .getByRole("dialog")
      .getByRole("button", { name: "Create", exact: true })
      .click();
    await expect(page.getByRole("dialog")).toBeHidden();

    await page.getByRole("tab", { name: "Configuration" }).click();
    const configurationForm = page.getByRole("form", {
      name: "Configuration",
    });
    const saveButton = configurationForm.getByRole("button", { name: "Save" });
    await expect(saveButton).toBeDisabled();
    await configurationForm
      .getByRole("checkbox", { name: "Allow registration" })
      .check();
    await configurationForm.getByLabel("Default TTL (seconds)").fill("7200");
    await configurationForm.getByLabel("Maximum TTL (seconds)").fill("90000");
    await configurationForm
      .getByRole("checkbox", { name: "Allow HTTP loopback redirect URIs" })
      .check();
    await saveButton.click();
    await expect(page.getByText("All changes saved")).toBeVisible();
    await expect(saveButton).toBeDisabled();

    await page.reload();
    await page.getByText("Current tenant").click();
    await page
      .getByRole("button", { name: new RegExp(`Playwright ${suffix}`) })
      .click();
    await page
      .getByRole("button", {
        name: new RegExp(`Playwright application ${suffix}`),
      })
      .click();
    await page.getByRole("tab", { name: "Configuration" }).click();
    await expect(
      configurationForm.getByLabel("Default TTL (seconds)"),
    ).toHaveValue("7200");
    await expect(
      configurationForm.getByLabel("Maximum TTL (seconds)"),
    ).toHaveValue("90000");
    await expect(
      configurationForm.getByRole("checkbox", { name: "Allow registration" }),
    ).toBeChecked();
    await expect(
      configurationForm.getByRole("checkbox", {
        name: "Allow HTTP loopback redirect URIs",
      }),
    ).toBeChecked();
  });
});
