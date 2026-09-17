import { defineConfig } from 'vite';
import { viteSingleFile } from 'vite-plugin-singlefile';
import { nodePolyfills } from 'vite-plugin-node-polyfills';

export default defineConfig({
  plugins: [
    viteSingleFile(),
    nodePolyfills({
      include: ['path', 'fs', 'util', 'process'],
      globals: {
        Buffer: true,
        global: true,
        process: true,
      },
    }),
  ],
  define: {
    '__dirname': '"/"',
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
  }
});
