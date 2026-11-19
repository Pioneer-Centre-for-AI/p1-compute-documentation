import { STATUS_LABELS, getRfcs, rfcTitle } from '#lib/rfcs.ts';
import { SITE_NAME, SITE_URL } from '#lib/site.ts';
import type { RequestHandler } from './$types';

export const prerender = true;

function escape(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

export const GET: RequestHandler = () => {
  const rfcs = getRfcs().toSorted(
    (a, b) => b.meta.updated.localeCompare(a.meta.updated) || b.number.localeCompare(a.number)
  );
  const newest = rfcs[0]?.meta.updated ?? '2026-11-06';

  const entries = rfcs.map((rfc) => {
    const summary = `${STATUS_LABELS[rfc.meta.status]}. ${rfc.meta.description}`;
    const authors = rfc.meta.authors.map((name) => `    <author><name>${escape(name)}</name></author>`).join('\n');
    return `  <entry>
    <title>${escape(rfcTitle(rfc))}</title>
    <id>${SITE_URL}${rfc.href}</id>
    <link href="${SITE_URL}${rfc.href}" />
    <updated>${rfc.meta.updated}T00:00:00Z</updated>
${authors}
    <summary>${escape(summary)}</summary>
  </entry>`;
  });

  const xml = `<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>${escape(`RFCs · ${SITE_NAME}`)}</title>
  <id>${SITE_URL}/rfc/</id>
  <link href="${SITE_URL}/rfc/" />
  <link rel="self" href="${SITE_URL}/rfc/feed.xml" />
  <updated>${newest}T00:00:00Z</updated>
${entries.join('\n')}
</feed>
`;

  return new Response(xml, {
    headers: { 'content-type': 'application/atom+xml; charset=utf-8' }
  });
};
