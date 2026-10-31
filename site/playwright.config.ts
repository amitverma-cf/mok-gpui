import { defineConfig, devices } from "@playwright/test";

// Runs against the built site; the gallery must be built for this origin.
export default defineConfig({
  testDir: "e2e",
  timeout: 60_000,
  fullyParallel: true,
  reporter: [["list"]],
  use: { baseURL: "http://localhost:4173", trace: "retain-on-failure" },
  webServer: { command: "pnpm preview --port 4173 --strictPort", url: "http://localhost:4173", reuseExistingServer: true },
  projects: [
    { name: "desktop", use: { ...devices["Desktop Chrome"] } },
    { name: "firefox", use: { ...devices["Desktop Firefox"] } },
    { name: "phone", use: { ...devices["Pixel 7"] } },
  ],
});
