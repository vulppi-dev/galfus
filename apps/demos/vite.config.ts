import { defineConfig, searchForWorkspaceRoot } from 'vite';

export default defineConfig({
  root: '.',
  assetsInclude: ['**/*.wasm'],
  optimizeDeps: {
    exclude: ['@galfus/transport-browser']
  },
  server: {
    fs: {
      allow: [searchForWorkspaceRoot(process.cwd())]
    }
  }
});
