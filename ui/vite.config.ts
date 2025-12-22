/// <reference types="vitest" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      "/api": {
        target: "http://localhost:3000",
        changeOrigin: true,
        rewrite: (path: string) => path.replace(/^\/api/, ""),
      },
      "/ws": {
        target: "ws://localhost:3000",
        changeOrigin: true,
        ws: true,
        rewrite: (path: string) => path,
      },
    },
  },
  test: {
    browser: {
      provider: "playwright", // or 'webdriverio'
      enabled: true,
      // at least one instance is required
      instances: [{ browser: "chromium" }],
      headless: true,
      viewport: { width: 1920, height: 1080 },
    },
  },
});
