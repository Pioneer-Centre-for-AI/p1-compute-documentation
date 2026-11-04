// /gallery/index.json: the published examples, for `aicentre gallery`.
import { getGallery } from '$lib/gallery';
import { hasFiles } from '$lib/server/gallery-files';
import { SITE_URL } from '$lib/site';
import type { RequestHandler } from './$types';

export const prerender = true;

export const GET: RequestHandler = () => {
  const examples = getGallery()
    .filter((e) => !e.draft)
    .map((e) => ({
      code: e.code,
      slug: e.slug,
      title: e.meta.title,
      summary: e.meta.summary,
      providers: e.meta.clusters,
      page: `${SITE_URL}${e.href}`,
      files: hasFiles(e.slug) ? `${SITE_URL}/gallery/${e.slug}.tar.gz` : null,
      commands: e.meta.commands ?? []
    }));
  return new Response(JSON.stringify({ examples }, null, 2) + '\n', { headers: { 'content-type': 'application/json' } });
};
