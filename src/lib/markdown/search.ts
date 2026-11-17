import { LANDING_DESCRIPTION } from '#lib/site.ts';
import { renderTwin, twinMeta, twinPaths } from './twins';

export type SearchEntry = {
  kind: 'page' | 'section' | 'requirement';
  title: string;
  section?: string;
  text: string;
  body: string;
  href: string;
};

const SNIPPET = 160;
const BODY = 600;

function plain(markdown: string): string {
  return markdown
    .replace(/!\[[^\]]*\]\([^)]*\)/g, '')
    .replace(/\[([^\]]*)\]\([^)]*\)/g, '$1')
    .replace(/[*`>]/g, '')
    .replace(/\s+/g, ' ')
    .trim();
}

function slugify(text: string, seen: Map<string, number>): string {
  const base = text
    .toLowerCase()
    .replace(/[^\p{L}\p{N}\p{M} _-]/gu, '')
    .replace(/ /g, '-');
  const count = seen.get(base) ?? 0;
  seen.set(base, count + 1);
  return count ? `${base}-${count}` : base;
}

function pageEntries(path: string): SearchEntry[] {
  const markdown = renderTwin(path);
  if (!markdown) return [];
  const { title, description } = twinMeta(path);
  const href = `/${path}/`;
  const entries: SearchEntry[] = [{ kind: 'page', title, text: description, body: description, href }];
  const seen = new Map<string, number>();
  let section: SearchEntry | null = null;
  let fenced = false;

  for (const line of markdown.split('\n')) {
    if (line.startsWith('```')) fenced = !fenced;
    if (fenced || line.startsWith('```')) continue;

    const heading = /^#{2,3}\s+(.+)$/.exec(line);
    if (heading) {
      const name = plain(heading[1]);
      section = { kind: 'section', title, section: name, text: '', body: '', href: `${href}#${slugify(name, seen)}` };
      entries.push(section);
      continue;
    }

    const requirement = /^\|\s*((?:RD|SL)\d+)\s*\|\s*(.+?)\s*\|\s*$/.exec(line);
    if (requirement && /\b(MUST|SHOULD|MAY)\b/.test(requirement[2])) {
      entries.push({
        kind: 'requirement',
        title: requirement[1],
        section: title,
        text: plain(requirement[2]),
        body: plain(requirement[2]),
        href: section?.href ?? href
      });
      continue;
    }

    if (!section || !line.trim() || /^\|[\s|:-]+\|$/.test(line)) continue;
    const words = plain(line.replace(/^\s*(?:[-*]|\d+\.)\s+/, '').replace(/\|/g, ' '));
    if (section.text.length < SNIPPET && !line.startsWith('|')) {
      section.text = `${section.text} ${words}`.trim().slice(0, SNIPPET);
    }
    if (section.body.length < BODY) {
      section.body = `${section.body} ${words}`.trim().slice(0, BODY);
    }
  }
  return entries;
}

export function searchIndex(): SearchEntry[] {
  return [
    { kind: 'page', title: 'Home', text: LANDING_DESCRIPTION, body: LANDING_DESCRIPTION, href: '/' },
    ...twinPaths.flatMap(pageEntries)
  ];
}
