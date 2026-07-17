import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import { mdsvex } from 'mdsvex';
import rehypeSlug from 'rehype-slug';
import rehypeExternalLinks from 'rehype-external-links';
import { renderCodeBlock } from './src/lib/markdown/highlight.js';

/**
 * mdsvex highlighter emitting the same chrome as CodeBlock.svelte.
 * Supports a filename header via the fence metastring: ```bash file=train.sh
 */
async function highlighter(code, lang, metastring) {
  const file = /(?:^|\s)file=(\S+)/.exec(metastring ?? '')?.[1];
  const html = await renderCodeBlock(code, lang ?? 'text', file);
  // JSON.stringify safely escapes braces, backticks, and newlines for the Svelte template.
  return `{@html ${JSON.stringify(html)}}`;
}

/** @type {import('@sveltejs/kit').Config} */
const config = {
  extensions: ['.svelte', '.svx'],
  preprocess: [
    mdsvex({
      extensions: ['.svx'],
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
      fallback: undefined,
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
