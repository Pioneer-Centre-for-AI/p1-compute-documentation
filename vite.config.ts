import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';
import { mdsvex } from 'mdsvex';
import { readdirSync } from 'node:fs';
import rehypeExternalLinks from 'rehype-external-links';
import rehypeSlug from 'rehype-slug';
import { defineConfig } from 'vite';
import { renderCodeBlock } from './src/lib/markdown/highlight.js';
import { tableScroll } from './src/lib/markdown/tableScroll.js';
import { urlScheme } from './src/lib/markdown/urlScheme.js';

const galleryDirs = readdirSync(new URL('./pkgs/gallery', import.meta.url), { withFileTypes: true })
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name);

async function highlighter(code: string, lang?: string | null, metastring?: string | null) {
  const file = /(?:^|\s)file=(\S+)/.exec(metastring ?? '')?.[1];
  const html = await renderCodeBlock(code, lang ?? 'text', file);
  return `{@html ${JSON.stringify(html)}}`;
}

export default defineConfig({
  plugins: [
    tailwindcss(),
    sveltekit({
      extensions: ['.svelte', '.svx'],
      preprocess: [
        mdsvex({
          extensions: ['.svx'],
          remarkPlugins: [urlScheme],
          rehypePlugins: [
            rehypeSlug,
            tableScroll,
            [rehypeExternalLinks, { target: '_blank', rel: ['noopener', 'noreferrer'] }]
          ],
          highlight: { highlighter },
          smartypants: false
        }),
        vitePreprocess()
      ],
      adapter: adapter({
        pages: 'build',
        assets: 'build',
        fallback: '404.html',
        strict: true
      }),
      paths: {
        base: ''
      },
      version: {
        pollInterval: 0
      }
    })
  ],
  define: { __GALLERY_DIRS__: JSON.stringify(galleryDirs) }
});
