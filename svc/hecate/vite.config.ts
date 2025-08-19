import MobxManager from '@lomray/react-mobx-manager/plugins/vite/index';
import SsrBoost from '@lomray/vite-ssr-boost/plugin';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

// https://vitejs.dev/config/
export default defineConfig({
  root: 'src',
  publicDir: '../public',
  envDir: '../',
  build: {
    outDir: '../build',
  },
  server: {
    host: true,
    port: 5173,
    strictPort: true,
    open: false, // Don't auto-open browser
    cors: true,
    hmr: {
      overlay: true,
    },
  },
  logLevel: 'info',
  clearScreen: false,
  plugins: [
    SsrBoost({
      logLevel: 'info',
    }), 
    react(), 
    MobxManager()
  ],
  // Enhanced logging for development
  customLogger: {
    info: (msg) => {
      const timestamp = new Date().toISOString();
      console.log(`🌐 [${timestamp}] ℹ️  Vite: ${msg}`);
    },
    warn: (msg) => {
      const timestamp = new Date().toISOString();
      console.log(`🌐 [${timestamp}] ⚠️  Vite: ${msg}`);
    },
    error: (msg) => {
      const timestamp = new Date().toISOString();
      console.log(`🌐 [${timestamp}] ❌ Vite: ${msg}`);
    },
  },
});
