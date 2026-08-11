// llms-full.txt — every page's markdown twin concatenated into one file, for
// pasting whole into a model context.
//
// Same renderer as the `.md` endpoint, so the two can never disagree. Pages
// appear in MARKDOWN_PAGES order, each preceded by its canonical URL so a model
// can cite the page a passage came from.
import { renderTwin, twinPaths } from '$lib/markdown/twins';
import { SITE_NAME, SITE_SUMMARY, SITE_URL } from '$lib/site';
import type { RequestHandler } from './$types';

export const prerender = true;

export const GET: RequestHandler = () => {
  const documents = twinPaths.map((path) => {
    // renderTwin only returns null for a path with no source, and twinPaths is
    // asserted against the sources on disk at import time.
    const markdown = renderTwin(path) ?? '';
    // Site-relative links resolve fine in a twin served from its own URL, but
    // this file gets pasted into a context with no base URL, so absolutize.
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
