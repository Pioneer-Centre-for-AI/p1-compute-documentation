// Route paths that have a raw markdown twin at `<path>.md`.
//
// Written out rather than globbed because this list is imported by the root
// layout, which runs in the browser: globbing the .svx sources here would ship
// every page's source to the client. The `.md` endpoint asserts this list
// matches the actual set of .svx pages, so drift fails the build instead of
// silently advertising a URL that 404s.
export const MARKDOWN_PAGES = [
  'about',
  'clusters',
  'clusters/dtu',
  'clusters/gefion',
  'clusters/ngc',
  'ecosystem',
  'events',
  'events/superagent-d3a',
  'forms',
  'learn',
  'privacy',
  'terms'
];

const set = new Set(MARKDOWN_PAGES);

/** `/about/` -> `/about.md`, or null when the page has no markdown twin. */
export function markdownHref(pathname: string): string | null {
  const path = pathname.replace(/^\/|\/$/g, '');
  return set.has(path) ? `/${path}.md` : null;
}
