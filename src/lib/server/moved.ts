// Sections that moved to a new path, and the page an old link lands on.
//
// The site is static on GitHub Pages, so no server can answer an old URL with
// a redirect. Each old path is prerendered as a small HTML page that sends
// the reader on instead. SvelteKit's own redirect stub would do that too, but
// it drops the #fragment, and old links often point at a heading.

import { SITE_URL } from '$lib/site';

/** Old first path segment -> new one. */
export const MOVED: Record<string, string> = {
  clusters: 'providers'
};

/** `clusters/dtu` -> `providers/dtu`; anything that did not move comes back unchanged. */
export function currentPath(path: string): string {
  const [head, ...rest] = path.split('/');
  return head in MOVED ? [MOVED[head], ...rest].join('/') : path;
}

/** Every old path that maps onto one of `paths`, for prerender entries. */
export function oldPaths(paths: readonly string[]): string[] {
  return Object.entries(MOVED).flatMap(([from, to]) =>
    paths.filter((p) => p === to || p.startsWith(`${to}/`)).map((p) => from + p.slice(to.length))
  );
}

/** An HTML page that forwards to `target`, keeping the query and the #fragment. */
export function movedPage(target: string): Response {
  const href = JSON.stringify(target);
  const html = `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Moved to ${target}</title>
<link rel="canonical" href="${SITE_URL}${target}">
<meta name="robots" content="noindex">
<script>location.replace(${href} + location.search + location.hash);</script>
<meta http-equiv="refresh" content="0; url=${target}">
</head>
<body>
<p>This page has moved to <a href="${target}">${SITE_URL}${target}</a>.</p>
</body>
</html>
`;
  // Exactly 'text/html': SvelteKit only saves a prerendered endpoint as
  // <path>/index.html for that type, and the charset is in the <meta> above.
  return new Response(html, { headers: { 'content-type': 'text/html' } });
}
