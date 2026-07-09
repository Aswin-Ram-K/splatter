import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';
import fs from 'fs';

// Vite plugin to copy ghostty-vt.wasm to dist during build
function copyGhosttyWasm() {
  return {
    name: 'copy-ghostty-wasm',
    closeBundle() {
      const src = path.resolve(__dirname, 'node_modules/ghostty-web/ghostty-vt.wasm');
      const dst = path.resolve(__dirname, 'dist/wasm/ghostty-vt.wasm');
      fs.mkdirSync(path.dirname(dst), { recursive: true });
      fs.copyFileSync(src, dst);
    },
  };
}

export default defineConfig({
  plugins: [react(), copyGhosttyWasm()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 5173,
    strictPort: true,
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          // Split ghostty-web into its own chunk
          'ghostty': ['ghostty-web'],
        },
      },
    },
  },
});
