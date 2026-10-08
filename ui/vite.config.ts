import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  base: './',  // relative paths so the built bundle works under file:// (Electron production)
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://localhost:19270',
        changeOrigin: true,
      },
    },
  },
});
