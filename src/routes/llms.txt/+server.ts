// llms.txt — an index of the site for language models, per llmstxt.org.
//
// The convention is an H1 name, a blockquote summary, then sections of links.
// Every link here points at the `.md` twin rather than the HTML page, so a
// model that follows one gets clean markdown instead of a Svelte-rendered DOM.
//
// Sections mirror src/lib/nav.ts, so a page added to the nav is indexed here
// without a second edit. Pages deliberately kept out of the nav (Terms,
// Privacy) land in `## Optional`, which the spec defines as safe to skip.
import { getNav, isGroup } from '$lib/nav';
import { twinMeta, twinPaths } from '$lib/markdown/twins';
import { LANDING_DESCRIPTION, SITE_NAME, SITE_SUMMARY, SITE_URL } from '$lib/site';
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

  for (const entry of getNav()) {
    if (!isGroup(entry)) continue;
    const paths = entry.items
      .map((item) => toPath(item.href))
      .filter((path) => twinPaths.includes(path) && !indexed.has(path));
    if (!paths.length) continue;
    paths.forEach((path) => indexed.add(path));
    sections.push(`## ${entry.label}\n\n${paths.map(link).join('\n')}`);
  }

  // The landing page is a +page.svelte, so it has no twin and no nav entry.
  // Link the HTML rather than leave the site's front door out of the index.
  sections.unshift(`## Start here\n\n- [${SITE_NAME}](${SITE_URL}/): ${LANDING_DESCRIPTION}`);

  const rest = twinPaths.filter((path) => !indexed.has(path));
  if (rest.length) {
    sections.push(`## Optional\n\n${rest.map(link).join('\n')}`);
  }

  const body = `# ${SITE_NAME}

> ${SITE_SUMMARY}

Almost every page has a markdown twin at the same path with \`.md\` appended, and that is what the
links below point at; the home page is the one exception and links to its HTML.
${SITE_URL}/llms-full.txt is the whole site as a single file.

${sections.join('\n\n')}
`;

  return new Response(body, {
    headers: { 'content-type': 'text/plain; charset=utf-8' }
  });
};
