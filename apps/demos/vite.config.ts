import { defineConfig, searchForWorkspaceRoot } from 'vite';

export default defineConfig({
  root: '.',
  assetsInclude: ['**/*.wasm'],
  optimizeDeps: {
    exclude: ['@vulfram/transport-browser'],
  },
  server: {
    fs: {
      allow: [searchForWorkspaceRoot(process.cwd()), '/home/morbden/Projetos'],
    },
  },
});
