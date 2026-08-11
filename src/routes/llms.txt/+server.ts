// llms.txt — an index of the site for language models, per llmstxt.org.
//
// The convention is an H1 name, a blockquote summary, then sections of links.
// Every link here points at the `.md` twin rather than the HTML page, so a
// model that follows one gets clean markdown instead of a Svelte-rendered DOM.
//
// Sections mirror src/lib/nav.ts, so a page added to the nav is indexed here
// without a second edit. Pages deliberately kept out of the nav (Terms,
// Privacy) land in `## Optional`, which the spec defines as safe to skip.
import { isGroup, nav } from '$lib/nav';
import { twinMeta, twinPaths } from '$lib/markdown/twins';
import { SITE_NAME, SITE_SUMMARY, SITE_URL } from '$lib/site';
import type { RequestHandler } from './$types';

export const prerender = true;

/** `/clusters/dtu/` -> `clusters/dtu` */
function toPath(href: string): string {
  return href.replace(/^\/|\/$/g, '');
}

function link(path: string): string {
  const { title, description } = twinMeta(path);
  const suffix = description ? `: ${description}` : '';
  return `- [${title}](${SITE_URL}/${path}.md)${suffix}`;
}

export const GET: RequestHandler = () => {
  const indexed = new Set<string>();
  const sections: string[] = [];

  for (const entry of nav) {
    if (!isGroup(entry)) continue;
    const paths = entry.items
      .map((item) => toPath(item.href))
      .filter((path) => twinPaths.includes(path) && !indexed.has(path));
    if (!paths.length) continue;
    paths.forEach((path) => indexed.add(path));
    sections.push(`## ${entry.label}\n\n${paths.map(link).join('\n')}`);
  }

  const rest = twinPaths.filter((path) => !indexed.has(path));
  if (rest.length) {
    sections.push(`## Optional\n\n${rest.map(link).join('\n')}`);
  }

  const body = `# ${SITE_NAME}

> ${SITE_SUMMARY}

Every page on this site has a markdown twin at the same path with \`.md\` appended, which is what
the links below point at. ${SITE_URL}/llms-full.txt is the whole site as a single file.

${sections.join('\n\n')}
`;

  return new Response(body, {
    headers: { 'content-type': 'text/plain; charset=utf-8' }
  });
};
