/// <reference types="vitest/config" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import process from "node:process";
import { resolve } from "node:path";

const host = process.env.TAURI_DEV_HOST;

// Tauri beklentileri: sabit port, Rust hatalarını gizlememek için clearScreen kapalı,
// src-tauri izlenmez (cargo kendi değişikliklerini izler).
export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  build: {
    target: "es2022",
    sourcemap: false,
    // İki sayfa: ana uygulama ve hafif açılış penceresi.
    rollupOptions: {
      input: {
        main: resolve(import.meta.dirname, "index.html"),
        splash: resolve(import.meta.dirname, "splash.html"),
      },
    },
  },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./src/test/setup.ts"],
    css: false,
    include: ["src/**/*.test.{ts,tsx}"],
  },
});
