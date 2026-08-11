// Sitemap for hpc.aicentre.dk, prerendered alongside the pages themselves.
//
// MARKDOWN_PAGES is the hand-written list of .svx routes, which the `.md`
// endpoint already asserts against the files on disk. Reusing it here means a
// new page cannot appear in the sitemap without also passing that guard, and
// cannot be silently left out of it either.
import { MARKDOWN_PAGES } from '$lib/markdownPages';
import { SITE_URL } from '$lib/site';
import type { RequestHandler } from './$types';

export const prerender = true;

// The landing page is a +page.svelte rather than a .svx, so it is not in
// MARKDOWN_PAGES and is added here explicitly.
const paths = ['', ...MARKDOWN_PAGES.map((p) => `${p}/`)];

export const GET: RequestHandler = () => {
  const urls = paths
    .map((p) => `  <url>\n    <loc>${SITE_URL}/${p}</loc>\n  </url>`)
    .join('\n');

  const xml = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${urls}
</urlset>
`;

  return new Response(xml, {
    headers: { 'content-type': 'application/xml; charset=utf-8' }
  });
};
