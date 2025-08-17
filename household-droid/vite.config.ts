/// <reference types="vitest" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      "/query": {
        target: "http://localhost:8000", // Backend server
        changeOrigin: true, // Ensure the request appears to come from the frontend server
      },
      "/tutor": {
        target: "http://localhost:8000", // Backend server
        changeOrigin: true, // Ensure the request appears to come from the frontend server
      },
      "/token": {
        target: "http://localhost:8000", // Backend server
        changeOrigin: true, // Ensure the request appears to come from the frontend server
      },
      "/session": {
        target: "http://localhost:8000", // Backend server
        changeOrigin: true, // Ensure the request appears to come from the frontend server
      },
      "/users": {
        target: "http://localhost:8000", // Backend server
        changeOrigin: true, // Ensure the request appears to come from the frontend server
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
    },
  },
});
