import MobxManager from '@lomray/react-mobx-manager/plugins/vite/index';
import SsrBoost from '@lomray/vite-ssr-boost/plugin';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';
import path from 'path';

// https://vitejs.dev/config/
export default defineConfig({
  root: 'src',
  publicDir: '../public',
  envDir: '../',
  build: {
    outDir: '../build',
  },
  resolve: {
    alias: {
      '@assets': path.resolve(__dirname, 'src/assets'),
      '@components': path.resolve(__dirname, 'src/common/components'),
      '@helpers': path.resolve(__dirname, 'src/common/helpers'),
      '@services': path.resolve(__dirname, 'src/common/services'),
      '@providers': path.resolve(__dirname, 'src/common/providers'),
      '@constants': path.resolve(__dirname, 'src/constants'),
      '@interfaces': path.resolve(__dirname, 'src/interfaces'),
      '@pages': path.resolve(__dirname, 'src/pages'),
      '@routes': path.resolve(__dirname, 'src/routes'),
    },
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
    warnOnce: (msg) => {
      const timestamp = new Date().toISOString();
      console.log(`🌐 [${timestamp}] ⚠️  Vite: ${msg}`);
    },
    error: (msg) => {
      const timestamp = new Date().toISOString();
      console.log(`🌐 [${timestamp}] ❌ Vite: ${msg}`);
    },
    clearScreen: () => {},
    hasWarned: false,
    hasErrorLogged: () => false,
  },
});
