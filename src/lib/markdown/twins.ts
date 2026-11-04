import { getClusters, getClusterDecisionEntries, computeTiers } from '$lib/content';
import { GALLERY_INTRO, GALLERY_SOURCE, getGallery } from '$lib/gallery';
import {
  getEvents,
  superAgentAtAGlance,
  superAgentCollaborations,
  superAgentImpactBoard,
  superAgentOrganizers,
  superAgentProgramme
} from '$lib/events';
import { FORMS } from '$lib/links';
import { MARKDOWN_PAGES } from '$lib/markdownPages';
import { URLS } from '$lib/urls';
import { markdownTable, svxToMarkdown } from './toMarkdown';

const sources = import.meta.glob(['/src/routes/**/+page.svx', '!/src/routes/gallery/[(]drafts[)]/**'], {
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

export const twinPaths = MARKDOWN_PAGES;

const BLOCKS: Record<string, () => Record<string, string>> = {
  about: () => ({
    'tier-table': markdownTable(
      ['Tier', 'Organization level', 'Description'],
      computeTiers.map((t) => [
        t.tier,
        t.level,
        t.description.replace(/<a href="([^"]+)">([^<]+)<\/a>/g, '[$2]($1)')
      ])
    )
  }),
  providers: () => ({
    'decision-table': markdownTable(
      ['If you need', 'Go to', 'Why'],
      getClusterDecisionEntries().map((d) => {
        const label = d.cluster.meta.navLabel ?? d.cluster.meta.title;
        return [d.need, `[${label}](${d.cluster.href})`, d.why];
      })
    ),
    guidance: `**Guidance:** unsure which fits? [Tell us about your workload](${URLS.surveyPage}) and the compute coordinator will help you find and use the right one. Optional: you can go straight to a provider page.`,
    'cluster-cards': getClusters()
      .map((c) => `- [${c.meta.title}](${c.href}): ${c.meta.description}`)
      .join('\n')
  }),
  survey: () => ({
    embed: `**Form:** [Your Compute Needs survey](${URLS.survey})`
  }),
  gallery: () => ({
    'gallery-hero':
      `# Gallery\n\n${GALLERY_INTRO}\n\n` +
      `Have a workflow others could use? Add it in a [pull request](${GALLERY_SOURCE}), ` +
      `or [tell the compute coordinator](${URLS.surveyPage}).`,
    'gallery-list': getGallery()
      .map(
        (e) =>
          `- [${e.code}: ${e.meta.title}](${e.href}): ${e.meta.summary} Tags: ${e.meta.tags.join(', ')}.`
      )
      .join('\n')
  }),
  forms: () => ({
    'forms-table': markdownTable(
      ['Form', 'Purpose', 'Provider'],
      FORMS.map((f) => [`[${f.title}](${f.url})`, f.purpose, f.provider])
    )
  }),
  events: () => ({
    'event-cards': getEvents()
      .map((e) => `- [${e.meta.title}](${e.href}) (${e.meta.dateLabel}): ${e.meta.description}`)
      .join('\n')
  }),
  'events/superagent-d3a': () => ({
    'about-cards': superAgentAtAGlance
      .map((p) => `### ${p.label}\n\n${p.items.map((i) => `- ${i}`).join('\n')}`)
      .join('\n\n'),
    programme: markdownTable(
      ['Time', 'Session', 'Speaker'],
      superAgentProgramme.map((row) => {
        const name = row.speakerUrl ? `[${row.speaker}](${row.speakerUrl})` : row.speaker;
        const session = row.link ? `${row.session} ([${row.link.label}](${row.link.href}))` : row.session;
        return [row.time, session, row.affiliation ? `${name}, ${row.affiliation}` : name];
      })
    ),
    'impact-board': `**Impact board:** [low, medium, and high impact issues](${superAgentImpactBoard.href})`,
    organizers: superAgentOrganizers
      .map((p) => `- [${p.name}](${p.profileUrl}), ${p.institution}: ${p.email}`)
      .join('\n'),
    collaborations: superAgentCollaborations
      .map((c) => `- [${c.name}](${c.href}): ${c.blurb}`)
      .join('\n')
  })
};

export function renderTwin(path: string): string | null {
  const source = pages.get(path);
  if (!source) return null;

  const cluster = getClusters().find((c) => c.href === `/${path}/`);
  const event = getEvents().find((e) => e.href === `/${path}/`);
  const example = getGallery().find((e) => e.href === `/${path}/`);

  return svxToMarkdown(source, {
    title: cluster?.meta.title ?? event?.meta.title ?? example?.meta.title,
    providerDocs: cluster?.meta.providerDocs,
    quotas: cluster?.meta.quotas,
    hardware: cluster?.meta.hardware,
    blocks: BLOCKS[path]?.()
  });
}

export function twinMeta(path: string): { title: string; description: string } {
  const cluster = getClusters().find((c) => c.href === `/${path}/`);
  const event = getEvents().find((e) => e.href === `/${path}/`);
  const example = getGallery().find((e) => e.href === `/${path}/`);
  const fromRegistry = cluster?.meta ?? event?.meta ?? example?.meta;
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
    description: description?.replace(/\s+/g, ' ').trim() ?? ''
  };
}
