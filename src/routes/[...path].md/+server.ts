// Raw markdown twin of every .svx page: append `.md` to a page URL to get a
// plain-text version that an LLM or a script can parse directly.
//
//   /about/           ->  /about.md
//   /clusters/dtu/    ->  /clusters/dtu.md
//
// SvelteKit compiles the `[...path].md` segment to `([^]*?)\.md`, so a single
// route covers nested pages. Endpoints default to trailingSlash 'never', which
// means this does not inherit the root layout's 'always' and `/about.md`
// resolves without a redirect.
import { error } from '@sveltejs/kit';
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
import { markdownTable, svxToMarkdown } from '$lib/markdown/toMarkdown';
import type { EntryGenerator, RequestHandler } from './$types';

export const prerender = true;

// Raw sources, keyed by route path. adapter-static runs with strict: true and
// no fallback, and the crawler cannot discover an unlinked endpoint, so the
// same glob feeds both the entry list and the response body.
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

export const entries: EntryGenerator = () => [...pages.keys()].map((path) => ({ path }));

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

export const GET: RequestHandler = ({ params }) => {
  const path = params.path;
  const source = pages.get(path);
  if (!source) error(404, `No markdown source for /${path}`);

  // Cluster and event pages get their H1 from the layout rather than the .svx
  // body, and cluster pages their hardware table too, so both are reattached here.
  const cluster = clusters.find((c) => c.href === `/${path}/`);
  const event = events.find((e) => e.href === `/${path}/`);

  const markdown = svxToMarkdown(source, {
    title: cluster?.meta.title ?? event?.meta.title,
    providerDocs: cluster?.meta.providerDocs,
    hardware: cluster?.meta.hardware,
    blocks: BLOCKS[path]?.()
  });

  return new Response(markdown, {
    headers: { 'content-type': 'text/markdown; charset=utf-8' }
  });
};
