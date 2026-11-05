import { error } from '@sveltejs/kit';
import { getClusters } from '#lib/content.ts';
import { movedPage } from '#lib/server/moved.ts';
import type { EntryGenerator, RequestHandler } from './$types';

export const prerender = true;
export const trailingSlash = 'always';

export const entries: EntryGenerator = () => getClusters().map((c) => ({ slug: c.slug }));

export const GET: RequestHandler = ({ params }) => {
  const cluster = getClusters().find((c) => c.slug === params.slug);
  if (!cluster) error(404, `No provider at /clusters/${params.slug}/`);
  return movedPage(cluster.href);
};
