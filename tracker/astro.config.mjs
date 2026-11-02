// @ts-check
import { defineConfig } from 'astro/config';

// Ships HTML and CSS only: no integrations, no islands, no client directives.
export default defineConfig({
  build: { format: 'directory' },
  compressHTML: true,
  markdown: {
    shikiConfig: { theme: 'github-dark' },
  },
});
