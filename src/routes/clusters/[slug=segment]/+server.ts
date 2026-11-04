// /clusters/<slug>/ became /providers/<slug>/. One forwarding page per
// provider, so a provider added later gets its old-style URL too.
import { error } from '@sveltejs/kit';
import { getClusters } from '$lib/content';
import { movedPage } from '$lib/server/moved';
import type { EntryGenerator, RequestHandler } from './$types';

export const prerender = true;
export const trailingSlash = 'always';

export const entries: EntryGenerator = () => getClusters().map((c) => ({ slug: c.slug }));

export const GET: RequestHandler = ({ params }) => {
  const cluster = getClusters().find((c) => c.slug === params.slug);
  if (!cluster) error(404, `No provider at /clusters/${params.slug}/`);
  return movedPage(cluster.href);
};
