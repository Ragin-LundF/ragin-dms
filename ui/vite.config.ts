/// <reference types="vitest/config" />
import {defineConfig} from "vite";
import react from "@vitejs/plugin-react";

// Tauri expects a fixed port and serves the built files from ui/dist. `--mode mock` (or no Tauri around the page)
// answers every command from `src/mocks`, for working in a browser and for headless checks.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 5191, strictPort: true },
  build: { target: "es2022", outDir: "dist" },
  test: { environment: "jsdom" },
});
