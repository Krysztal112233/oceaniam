import { expect, test } from "@playwright/test";

test.describe("real backend smoke", () => {
  test.skip(
    !process.env.OCEANIAM_E2E_ROOT_PASSWORD,
    "Set OCEANIAM_E2E_ROOT_PASSWORD for the isolated backend.",
  );

  test("authenticates through the real API and creates a tenant", async ({
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
  });
});
