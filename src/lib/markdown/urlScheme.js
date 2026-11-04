import { URLS } from '../urls.js';

const PREFIX = 'p1:';

/**
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
