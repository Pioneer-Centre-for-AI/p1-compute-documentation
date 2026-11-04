import { SITE_URL } from '$lib/site';

export const MOVED: Record<string, string> = {
  clusters: 'providers'
};

export function currentPath(path: string): string {
  const [head, ...rest] = path.split('/');
  return head in MOVED ? [MOVED[head], ...rest].join('/') : path;
}

export function oldPaths(paths: readonly string[]): string[] {
  return Object.entries(MOVED).flatMap(([from, to]) =>
    paths.filter((p) => p === to || p.startsWith(`${to}/`)).map((p) => from + p.slice(to.length))
  );
}

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
  return new Response(html, { headers: { 'content-type': 'text/html' } });
}
