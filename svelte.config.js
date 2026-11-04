import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import { mdsvex } from 'mdsvex';
import rehypeSlug from 'rehype-slug';
import rehypeExternalLinks from 'rehype-external-links';
import { renderCodeBlock } from './src/lib/markdown/highlight.js';
import { urlScheme } from './src/lib/markdown/urlScheme.js';

async function highlighter(code, lang, metastring) {
  const file = /(?:^|\s)file=(\S+)/.exec(metastring ?? '')?.[1];
  const html = await renderCodeBlock(code, lang ?? 'text', file);
  return `{@html ${JSON.stringify(html)}}`;
}

/** @type {import('@sveltejs/kit').Config} */
const config = {
  extensions: ['.svelte', '.svx'],
  preprocess: [
    mdsvex({
      extensions: ['.svx'],
      remarkPlugins: [urlScheme],
      rehypePlugins: [
        rehypeSlug,
        [rehypeExternalLinks, { target: '_blank', rel: ['noopener', 'noreferrer'] }]
      ],
      highlight: { highlighter },
      smartypants: false
    }),
    vitePreprocess()
  ],
  kit: {
    adapter: adapter({
      pages: 'build',
      assets: 'build',
      fallback: '404.html',
      strict: true
    }),
    paths: {
      base: ''
    },
    alias: {
      '@': 'src'
    }
  }
};

export default config;
