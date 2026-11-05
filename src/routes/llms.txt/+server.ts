import { GALLERY_HREF, getGallery } from '#lib/gallery.ts';
import { getNav, isGroup } from '#lib/nav.ts';
import { twinMeta, twinPaths } from '#lib/markdown/twins.ts';
import { LANDING_DESCRIPTION, SITE_NAME, SITE_SUMMARY, SITE_URL } from '#lib/site.ts';
import type { RequestHandler } from './$types';

export const prerender = true;

/** `/providers/dtu/` -> `providers/dtu` */
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

  const gallery = [GALLERY_HREF, ...getGallery().map((e) => e.href)]
    .map(toPath)
    .filter((path) => twinPaths.includes(path) && !indexed.has(path));
  gallery.forEach((path) => indexed.add(path));
  sections.push(`## Gallery\n\n${gallery.map(link).join('\n')}`);

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
