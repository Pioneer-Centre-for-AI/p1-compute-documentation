// remark plugin resolving `p1:<key>` link targets against src/lib/urls.js.
//
// Why this exists: in .svx, a raw `<a href={CONST}>` that starts a line
// terminates the enclosing paragraph, so the link and the rest of the sentence
// escape the <p>. Markdown links never do that, but markdown cannot hold a
// Svelte expression. Writing `[workload survey](p1:survey)` keeps the prose as
// plain markdown while URLs stay centralized.
//
// Runs before rehype-external-links, so resolved URLs still get target/rel.
import { URLS } from '../urls.js';

const PREFIX = 'p1:';

/**
 * Resolve a single `p1:key` target. Throws on an unknown key.
 * @param {string} target
 * @returns {string}
 */
export function resolveUrl(target) {
  const key = /** @type {keyof typeof URLS} */ (target.slice(PREFIX.length));
  const url = URLS[key];
  if (!url) {
    throw new Error(
      `Unknown p1: URL key "${key}". Known keys: ${Object.keys(URLS).join(', ')}`
    );
  }
  return url;
}

/** @param {any} node */
function walk(node) {
  if (node.type === 'link' && typeof node.url === 'string' && node.url.startsWith(PREFIX)) {
    node.url = resolveUrl(node.url);
  }
  for (const child of node.children ?? []) walk(child);
}

export function urlScheme() {
  return (/** @type {any} */ tree) => walk(tree);
}
