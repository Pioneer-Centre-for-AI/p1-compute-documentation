import { renderTwin, twinPaths } from '#lib/markdown/twins.ts';
import { SITE_NAME, SITE_SUMMARY, SITE_URL } from '#lib/site.ts';
import type { RequestHandler } from './$types';

export const prerender = true;

export const GET: RequestHandler = () => {
  const documents = twinPaths.map((path) => {
    const markdown = renderTwin(path) ?? '';
    const absolute = markdown.replace(/\]\(\/(?!\/)/g, `](${SITE_URL}/`);
    return `<!-- source: ${SITE_URL}/${path}/ -->\n\n${absolute}`;
  });

  const body = `# ${SITE_NAME}

> ${SITE_SUMMARY}

---

${documents.join('\n---\n\n')}`;

  return new Response(body, {
    headers: { 'content-type': 'text/plain; charset=utf-8' }
  });
};
