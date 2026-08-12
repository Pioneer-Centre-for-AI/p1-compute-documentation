// The markdown twin of every .svx page, and the metadata needed to index them.
//
// Three endpoints share this: `[...path].md` serves one twin, `llms-full.txt`
// concatenates all of them, and `llms.txt` lists them with titles and
// descriptions. Server-only — the glob below pulls every page source into the
// bundle, which must never reach the client.
import { clusters, clusterDecisionEntries, computeTiers } from '$lib/content';
import {
  events,
  superAgentAtAGlance,
  superAgentCollaborations,
  superAgentOrganizers,
  superAgentProgramme
} from '$lib/events';
import { FORMS } from '$lib/links';
import { MARKDOWN_PAGES } from '$lib/markdownPages';
import { URLS } from '$lib/urls';
import { markdownTable, svxToMarkdown } from './toMarkdown';

// Raw sources, keyed by route path. adapter-static runs with strict: true and
// no fallback, and the crawler cannot discover an unlinked endpoint, so the
// same glob feeds both the entry lists and the response bodies.
const sources = import.meta.glob('/src/routes/**/+page.svx', {
  query: '?raw',
  import: 'default',
  eager: true
}) as Record<string, string>;

const pages = new Map<string, string>(
  Object.entries(sources).map(([file, source]) => [
    file.replace(/^\/src\/routes\//, '').replace(/\/\+page\.svx$/, ''),
    source
  ])
);

// MARKDOWN_PAGES drives the rel=alternate links and the "View as Markdown"
// affordance. It is hand-written so the layout does not pull page sources into
// the client bundle, so guard it against drift here, where the real set lives.
const actual = [...pages.keys()].sort().join(',');
const advertised = [...MARKDOWN_PAGES].sort().join(',');
if (actual !== advertised) {
  throw new Error(
    `MARKDOWN_PAGES is out of date with the .svx pages on disk.\n` +
      `  on disk:     ${actual}\n` +
      `  advertised:  ${advertised}\n` +
      `Update src/lib/markdownPages.ts.`
  );
}

/** Every route path that has a `.md` twin, in MARKDOWN_PAGES order. */
export const twinPaths = MARKDOWN_PAGES;

// Index pages build their tables from a registry with a Svelte loop, which no
// source-level transform can follow. They mark those regions with
// `<!-- md:replace name -->` and the same registry renders them as markdown here.
const BLOCKS: Record<string, () => Record<string, string>> = {
  about: () => ({
    // The component renders cards or a table depending on width; markdown gets
    // the table. Descriptions carry inline <a> tags, rewritten to markdown here.
    'tier-table': markdownTable(
      ['Tier', 'Organization level', 'Description'],
      computeTiers.map((t) => [
        t.tier,
        t.level,
        t.description.replace(/<a href="([^"]+)">([^<]+)<\/a>/g, '[$2]($1)')
      ])
    )
  }),
  clusters: () => ({
    'decision-table': markdownTable(
      ['If you need', 'Go to', 'Why'],
      clusterDecisionEntries.map((d) => {
        const label = d.cluster.meta.navLabel ?? d.cluster.meta.title;
        return [d.need, `[${label}](${d.cluster.href})`, d.why];
      })
    ),
    guidance: `**Guidance:** unsure which fits, or want a second opinion? The compute coordinator is glad to help: [tell us about your workload](${URLS.surveyPage}). Entirely optional, and you are just as welcome to head straight to a cluster page.`,
    'cluster-cards': clusters
      .map((c) => `- [${c.meta.title}](${c.href}): ${c.meta.description}`)
      .join('\n')
  }),
  // The page embeds the form in an iframe, which is meaningless in markdown;
  // the twin gets the link instead.
  survey: () => ({
    embed: `**Form:** [Your Compute Needs survey](${URLS.survey})`
  }),
  forms: () => ({
    'forms-table': markdownTable(
      ['Form', 'Purpose', 'Provider'],
      FORMS.map((f) => [`[${f.title}](${f.url})`, f.purpose, f.provider])
    )
  }),
  events: () => ({
    'event-cards': events
      .map((e) => `- [${e.meta.title}](${e.href}) (${e.meta.dateLabel}): ${e.meta.description}`)
      .join('\n')
  }),
  'events/superagent-d3a': () => ({
    // Columns on the page; markdown has no columns, so each panel becomes a heading.
    'about-cards': superAgentAtAGlance
      .map((p) => `### ${p.label}\n\n${p.items.map((i) => `- ${i}`).join('\n')}`)
      .join('\n\n'),
    programme: markdownTable(
      ['Time', 'Session', 'Speaker'],
      superAgentProgramme.map((row) => {
        const name = row.speakerUrl ? `[${row.speaker}](${row.speakerUrl})` : row.speaker;
        return [row.time, row.session, row.affiliation ? `${name}, ${row.affiliation}` : name];
      })
    ),
    organizers: superAgentOrganizers
      .map((p) => `- [${p.name}](${p.profileUrl}), ${p.institution}: ${p.email}`)
      .join('\n'),
    collaborations: superAgentCollaborations
      .map((c) => `- [${c.name}](${c.href}): ${c.blurb}`)
      .join('\n')
  })
};

/** The markdown twin of one route path, or null when the path has no source. */
export function renderTwin(path: string): string | null {
  const source = pages.get(path);
  if (!source) return null;

  // Cluster and event pages get their H1 from the layout rather than the .svx
  // body, and cluster pages their hardware table too, so both are reattached here.
  const cluster = clusters.find((c) => c.href === `/${path}/`);
  const event = events.find((e) => e.href === `/${path}/`);

  return svxToMarkdown(source, {
    title: cluster?.meta.title ?? event?.meta.title,
    providerDocs: cluster?.meta.providerDocs,
    hardware: cluster?.meta.hardware,
    blocks: BLOCKS[path]?.()
  });
}

/**
 * Title and description for the llms.txt index. Cluster and event pages carry
 * both in validated frontmatter; the standalone pages pass them to `<Meta>`,
 * so read them back off that tag rather than keeping a second hand-written
 * list that could drift from what the page actually says.
 *
 * Read off the COMPONENT CALL, not the rendered head: these are page sources,
 * so `<title>` and `<meta name="description">` never appear in them. Matching
 * on those silently produced the route slug as a title and no description at
 * all, which is a valid llms.txt and a useless one.
 */
export function twinMeta(path: string): { title: string; description: string } {
  const cluster = clusters.find((c) => c.href === `/${path}/`);
  const event = events.find((e) => e.href === `/${path}/`);
  const fromRegistry = cluster?.meta ?? event?.meta;
  if (fromRegistry) {
    return { title: fromRegistry.title, description: fromRegistry.description };
  }

  const source = pages.get(path) ?? '';
  const tag = /<Meta\b[\s\S]*?\/>/.exec(source)?.[0] ?? '';
  const title = /\btitle="([^"]*)"/.exec(tag)?.[1]?.trim();
  const description = /\bdescription="([^"]*)"/.exec(tag)?.[1];
  if (!title) {
    throw new Error(
      `No <Meta title="…"> in src/routes/${path}/+page.svx, so llms.txt would index it as "${path}".`
    );
  }
  return {
    title,
    // Collapse the line wrapping the source uses inside the attribute.
    description: description?.replace(/\s+/g, ' ').trim() ?? ''
  };
}
