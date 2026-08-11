// Raw markdown twin of every .svx page: append `.md` to a page URL to get a
// plain-text version that an LLM or a script can parse directly.
//
//   /about/           ->  /about.md
//   /clusters/dtu/    ->  /clusters/dtu.md
//
// SvelteKit compiles the `[...path].md` segment to `([^]*?)\.md`, so a single
// route covers nested pages. Endpoints default to trailingSlash 'never', which
// means this does not inherit the root layout's 'always' and `/about.md`
// resolves without a redirect.
//
// The rendering itself lives in $lib/markdown/twins so that llms.txt and
// llms-full.txt produce byte-identical output from the same source.
import { error } from '@sveltejs/kit';
import { renderTwin, twinPaths } from '$lib/markdown/twins';
import type { EntryGenerator, RequestHandler } from './$types';

export const prerender = true;

export const entries: EntryGenerator = () => twinPaths.map((path) => ({ path }));

export const GET: RequestHandler = ({ params }) => {
  const markdown = renderTwin(params.path);
  if (!markdown) error(404, `No markdown source for /${params.path}`);

  return new Response(markdown, {
    headers: { 'content-type': 'text/markdown; charset=utf-8' }
  });
};
