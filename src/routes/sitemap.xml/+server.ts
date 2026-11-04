import { MARKDOWN_PAGES } from '$lib/markdownPages';
import { SITE_URL } from '$lib/site';
import type { RequestHandler } from './$types';

export const prerender = true;

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
