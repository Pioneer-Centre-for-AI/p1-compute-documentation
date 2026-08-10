// Shiki rendering for mdsvex fenced code blocks (build time, via svelte.config.js).
// Plain JS because svelte.config.js is loaded by Node.
import { createHighlighter } from 'shiki';

/** @type {Promise<import('shiki').Highlighter> | null} */
let highlighterPromise = null;

export function getHighlighter() {
  if (!highlighterPromise) {
    highlighterPromise = createHighlighter({
      themes: ['github-light', 'github-dark'],
      langs: ['bash', 'ini', 'text']
    });
  }
  return highlighterPromise;
}

/**
 * @param {string} code
 * @param {string} lang
 * @returns {Promise<string>}
 */
export async function renderCode(code, lang) {
  const hl = await getHighlighter();
  const langs = hl.getLoadedLanguages();
  const safeLang = langs.includes(/** @type {never} */ (lang)) ? lang : 'text';
  return hl.codeToHtml(code, {
    lang: safeLang,
    themes: { light: 'github-light', dark: 'github-dark' },
    defaultColor: false
  });
}

/** @param {string} s */
function escapeHtml(s) {
  return s.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;');
}

const FILE_ICON =
  '<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" /><polyline points="14,2 14,8 20,8" /></svg>';

const COPY_ICON =
  '<svg class="copy-icon-idle" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="9" y="9" width="13" height="13" rx="2" /><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" /></svg>';

const CHECK_ICON =
  '<svg class="copy-icon-done" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><polyline points="20,6 9,17 4,12" /></svg>';

// The button carries no event handler: this HTML is injected with {@html},
// so a delegated listener in the root layout wires every block at once.
const COPY_BUTTON = `<button type="button" class="copy-btn" data-copy aria-label="Copy code">${COPY_ICON}${CHECK_ICON}</button>`;

/**
 * Full code block chrome for fenced blocks in .svx files.
 * @param {string} code
 * @param {string} [lang]
 * @param {string} [file]
 * @returns {Promise<string>}
 */
export async function renderCodeBlock(code, lang = 'text', file) {
  const inner = await renderCode(code, lang);
  // The header bar is always rendered, even with nothing to label, so the copy
  // button has a row of its own. Absolutely positioning it over the code meant
  // it covered the first line on narrow screens, where the line scrolls under
  // it rather than reflowing.
  const label = file
    ? `${FILE_ICON}<span>${escapeHtml(file)}</span>`
    : lang && lang !== 'text'
      ? `<span class="opacity-60">${escapeHtml(lang)}</span>`
      : '';
  const header = `<div class="flex h-8 items-center gap-2 border-b border-slate-200 bg-slate-100 px-3 font-mono text-xs text-slate-700 dark:border-slate-700 dark:bg-slate-800/70 dark:text-slate-300">${label}</div>`;
  return `<div class="codeblock group not-prose relative my-3 overflow-hidden rounded-lg border border-slate-200 text-sm dark:border-slate-700">${header}${COPY_BUTTON}<div class="overflow-x-auto">${inner}</div></div>`;
}
