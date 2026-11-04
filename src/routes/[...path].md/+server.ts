import { error } from '@sveltejs/kit';
import { renderTwin, twinPaths } from '$lib/markdown/twins';
import { currentPath, oldPaths } from '$lib/server/moved';
import type { EntryGenerator, RequestHandler } from './$types';

export const prerender = true;

export const entries: EntryGenerator = () =>
  [...twinPaths, ...oldPaths(twinPaths)].map((path) => ({ path }));

export const GET: RequestHandler = ({ params }) => {
  const markdown = renderTwin(currentPath(params.path));
  if (!markdown) error(404, `No markdown source for /${params.path}`);

  return new Response(markdown, {
    headers: { 'content-type': 'text/markdown; charset=utf-8' }
  });
};
