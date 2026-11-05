import { error } from '@sveltejs/kit';
import { getGallery } from '#lib/gallery.ts';
import { exampleFiles, hasFiles } from '#lib/server/gallery-files.ts';
import { tarGz } from '#lib/server/tar.ts';
import type { EntryGenerator, RequestHandler } from './$types';

export const prerender = true;

const published = () => getGallery().filter((e) => !e.draft && hasFiles(e.slug));

export const entries: EntryGenerator = () => published().map((e) => ({ slug: e.slug }));

export const GET: RequestHandler = ({ params }) => {
  if (!published().some((e) => e.slug === params.slug)) error(404, 'No such example');
  return new Response(tarGz(params.slug, exampleFiles(params.slug)), { headers: { 'content-type': 'application/gzip' } });
};
