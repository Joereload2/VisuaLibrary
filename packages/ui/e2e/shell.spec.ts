import { expect, test } from "@playwright/test";

/**
 * Smoke E2E for the six MVP workstations (UI shell via Vite; no Tauri IPC).
 */
test.describe("Visual Library shell", () => {
  test("loads Library and exposes six primary stations", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("heading", { name: "Library" })).toBeVisible();

    const nav = page.getByRole("navigation", { name: "Estaciones de trabajo" });
    await expect(nav.getByRole("link", { name: "Factory" })).toBeVisible();
    await expect(nav.getByRole("link", { name: "Review" })).toBeVisible();
    await expect(nav.getByRole("link", { name: "Library" })).toBeVisible();
    await expect(nav.getByRole("link", { name: "Coverage" })).toBeVisible();
    await expect(nav.getByRole("link", { name: "Plans" })).toBeVisible();
    await expect(nav.getByRole("link", { name: "Settings" })).toBeVisible();
  });

  test("navigates to each primary station", async ({ page }) => {
    await page.goto("/");

    const stations = [
      { name: "Factory", heading: "Manual Factory" },
      { name: "Review", heading: "Review" },
      { name: "Library", heading: "Library" },
      { name: "Coverage", heading: "Coverage" },
      { name: "Plans", heading: "Plans" },
      { name: "Settings", heading: "Settings" },
    ] as const;

    for (const station of stations) {
      await page.getByRole("navigation").getByRole("link", { name: station.name }).click();
      await expect(page.getByRole("heading", { name: station.heading })).toBeVisible();
    }
  });
});
