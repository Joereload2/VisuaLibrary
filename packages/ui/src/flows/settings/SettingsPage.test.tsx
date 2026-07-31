import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SettingsPage } from "./SettingsPage";

vi.mock("../../shared/ipc/client", () => ({
  invokeGetAppPaths: vi.fn(async () => {
    throw new Error("no tauri");
  }),
  invokeGetSettings: vi.fn(async () => {
    throw new Error("no tauri");
  }),
  invokeSetMediaRoot: vi.fn(),
}));

describe("SettingsPage", () => {
  it("shows UI-only empty state when IPC is unavailable", async () => {
    render(<SettingsPage />);
    expect(
      await screen.findByText(/IPC no disponible/i, {}, { timeout: 3000 }),
    ).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Settings" })).toBeInTheDocument();
  });
});
