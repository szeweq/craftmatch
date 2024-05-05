import { defineConfig } from "vite";
import UnoCSS from "unocss/vite";
//import { svelte } from "@sveltejs/vite-plugin-svelte";
import extractorSvelte from "@unocss/extractor-svelte";
import { sveltekit } from '@sveltejs/kit/vite'

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [UnoCSS({extractors: [extractorSvelte()]}), sveltekit()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
