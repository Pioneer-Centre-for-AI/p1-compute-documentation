export const MARKDOWN_PAGES = [
  'about',
  'cli',
  'ecosystem',
  'events',
  'events/superagent-d3a',
  'forms',
  'gallery',
  'gallery/int-gpu',
  'gallery/rem-lgn',
  'gallery/xfr-dat',
  'learn',
  'privacy',
  'providers',
  'providers/dtu',
  'providers/gefion',
  'providers/ngc',
  'rfc',
  'rfc/0001',
  'rfc/0002',
  'rfc/all',
  'rfc/conventions',
  'survey',
  'terms'
];

const set = new Set(MARKDOWN_PAGES);

export function markdownHref(pathname: string): string | null {
  const path = pathname.replace(/^\/|\/$/g, '');
  return set.has(path) ? `/${path}.md` : null;
}
