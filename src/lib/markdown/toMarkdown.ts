// Turns a .svx page source into plain markdown for the `.md` twin of each page.
//
// The goal is a file an LLM or a script can read without tripping over Svelte:
// no <script>, no component tags, no layout wrappers, and absolute URLs
// everywhere. Fenced code blocks are protected before any tag stripping runs,
// otherwise shell placeholders like `User <your-username>_sftp` would be eaten
// as if they were HTML.
import * as links from '$lib/links';
import { QUOTA_GROUPS, type ClusterQuotaMeta } from '$lib/content';
import { URLS } from '$lib/urls';

type Meta = {
  /** H1 to prepend. Cluster pages get their title from the layout, not the source. */
  title?: string;
  /** Hardware rows to append, mirroring what HardwareSpec renders in HTML. */
  hardware?: { label: string; value: string }[];
  /** Provider documentation pointer, rendered by the layout rather than the source. */
  providerDocs?: { provider: string; url?: string; note?: string };
  /** Limits behind `<ClusterQuotas />`, which reads them from the cluster registry. */
  quotas?: ClusterQuotaMeta;
  /**
   * Markdown for regions the page builds with Svelte loops, keyed by the name
   * in its `<!-- md:replace NAME -->` marker. Index pages render their tables
   * from a registry, which does not survive a source-level transform, so the
   * caller supplies the same rows built from that same registry.
   */
  blocks?: Record<string, string>;
};

/** `<!-- md:replace name -->` ... `<!-- md:end -->` */
const REPLACE_REGION = /<!--\s*md:replace\s+([\w-]+)\s*-->[\s\S]*?<!--\s*md:end\s*-->/g;

/** Escape a cell so a stray pipe cannot break the table. */
function cell(value: string): string {
  return value.replace(/\|/g, '\\|');
}

/** Build a markdown table from a header row and body rows. */
export function markdownTable(headers: string[], rows: string[][]): string {
  const head = `| ${headers.map(cell).join(' | ')} |`;
  const rule = `| ${headers.map(() => '---').join(' | ')} |`;
  const body = rows.map((r) => `| ${r.map(cell).join(' | ')} |`);
  return [head, rule, ...body].join('\n');
}

const FENCE = /^```[\s\S]*?^```/gm;

/** Resolve `{SOME_CONST}` or a quoted literal from a component attribute. */
function resolveAttr(raw: string | undefined): string | undefined {
  if (!raw) return undefined;
  // Only unwrap when the whole attribute is one expression, so that a mixed
  // value like "{GEFION_SIGNUP_FORM_URL}&embed=true" falls through below.
  const expr = /^\{([^{}]*)\}$/.exec(raw.trim())?.[1]?.trim() ?? raw.trim();
  const known = (links as Record<string, unknown>)[expr];
  if (typeof known === 'string') return known;
  // Interpolated form, e.g. "{GEFION_SIGNUP_FORM_URL}&embed=true".
  const interpolated = expr.replace(/\{(\w+)\}/g, (m, name) => {
    const value = (links as Record<string, unknown>)[name];
    return typeof value === 'string' ? value : m;
  });
  return interpolated || undefined;
}

/** Pull `name="value"` or `name={expr}` out of a component tag. */
function attr(tag: string, name: string): string | undefined {
  const quoted = new RegExp(`\\b${name}="([^"]*)"`).exec(tag)?.[1];
  if (quoted !== undefined) return quoted;
  return new RegExp(`\\b${name}=(\\{[^}]*\\})`).exec(tag)?.[1];
}

/** A form the page embeds, rendered as a link line. */
function formLine(title: string, url: string | undefined, description?: string): string {
  const target = url ?? '';
  const suffix = description ? ` ${description}` : '';
  return `**Form:** [${title}](${target})${suffix}`;
}

/**
 * Components that wrap a form plus a fixed blurb. The .md twin keeps the link,
 * since that is the actionable part, and drops the presentational wrapper.
 */
const WRAPPERS: Record<string, () => string> = {
  AccessException: () =>
    formLine(
      'Access exception request',
      URLS.accessException,
      'For MSc students, research assistants, and similar roles who need cluster access with written approval from a responsible P1 co-lead or faculty member.'
    ),
  GpuAllowance: () =>
    formLine(
      'Planned GPU allocation',
      URLS.gpuAllowance,
      'Request a raised GPU limit for a run that also needs protecting from cancellation: a multi-day training, a deadline, or a workload that cannot checkpoint.'
    ),
  ComputeBudget: () =>
    formLine(
      'Investing your compute budget',
      URLS.computeBudget,
      'Advice on the split between cluster hardware, storage, cloud, and workstations.'
    ),
  NeedsCallout: () =>
    formLine(
      'Your compute needs survey',
      URLS.surveyPage,
      'Tell the P1 compute coordinator about your workload, needs, and expectations.'
    ),
  BookConsultation: () =>
    "**Book a consultation:** [pick a time in the compute coordinator's calendar]" +
    `(${URLS.booking}), in person or online.`,
  People: () => '_(Governance board members are listed on the web version of this page.)_',
  Citation: () =>
    '**Citing this cluster:** if the P1 DTU HPC contributed to your work, please cite the DTU ' +
    'Computing Center resources DOI, [10.48714/DTU.HPC.0001](https://doi.org/10.48714/DTU.HPC.0001). ' +
    'That covers help from the DTU DCC team or the P1 compute coordinator just as much as time ' +
    'spent running jobs.',
  LandscapeDiagram: () =>
    '_(Diagram: a researcher enters through P1, which consults and guides, and points on to university, national, and international HPCs.)_',
  ResearchNetworkDiagram: () =>
    '_(Diagram: forskningsnettet drawn as one high-bandwidth link, with the DTU, NGC, and Gefion facilities hosting the P1 partitions above it and the UCPH, SDU, Aarhus, and Computerome facilities below it.)_'
};

export function svxToMarkdown(source: string, meta: Meta = {}): string {
  let out = source;

  // 1. Frontmatter is layout metadata, not prose.
  out = out.replace(/^---\r?\n[\s\S]*?\r?\n---\r?\n/, '');

  // 2. Protect fenced code before any tag stripping.
  const fences: string[] = [];
  out = out.replace(FENCE, (block) => {
    fences.push(block);
    return `%%FENCE${fences.length - 1}%%`;
  });

  // 3. Swap Svelte-rendered regions for their markdown equivalent. Done before
  //    tag stripping so the loop markup inside never reaches the later passes.
  out = out.replace(REPLACE_REGION, (_m, name: string) => {
    const block = meta.blocks?.[name];
    if (block === undefined) {
      throw new Error(`Page uses <!-- md:replace ${name} --> but no block was supplied for it.`);
    }
    return block;
  });

  // 4. Drop Svelte and layout scaffolding.
  out = out.replace(/<script[\s\S]*?<\/script>/g, '');
  out = out.replace(/<svelte:head[\s\S]*?<\/svelte:head>/g, '');
  out = out.replace(/<\/?article\b[^>]*>/g, '');
  out = out.replace(/<p class="eyebrow[^"]*"[^>]*>[\s\S]*?<\/p>/g, '');
  out = out.replace(/<img\b[^>]*>/g, '');

  // 5. Aside becomes a blockquote, keeping its label.
  out = out.replace(/<Aside\b([^>]*)>([\s\S]*?)<\/Aside>/g, (_m, rawAttrs: string, body: string) => {
    const label = attr(rawAttrs, 'title') ?? attr(rawAttrs, 'type') ?? 'Note';
    const quoted = body
      .trim()
      .split('\n')
      .map((line) => (line.trim() ? `> ${line}` : '>'))
      .join('\n');
    return `> **${label}**\n>\n${quoted}`;
  });

  // 6. EmbedForm keeps its title, URL, and description.
  out = out.replace(/<EmbedForm\b[\s\S]*?\/>/g, (tag) => {
    const title = attr(tag, 'title') ?? 'Form';
    const url = resolveAttr(attr(tag, 'href')) ?? resolveAttr(attr(tag, 'src'));
    return formLine(title, url, attr(tag, 'description'));
  });

  // 6b. ClusterQuotas takes only a slug; the values live in the registry, so
  //     the caller supplies them the same way it supplies the hardware rows.
  out = out.replace(/<ClusterQuotas\b[^>]*\/>/g, () => {
    const quotas = meta.quotas;
    if (!quotas) return '';
    return QUOTA_GROUPS.map(
      (group) =>
        `**${group.title}**\n\n` +
        markdownTable(
          ['Limit', 'Value'],
          group.rows.map((r) => [r.label, quotas[r.key]])
        )
    ).join('\n\n');
  });

  // 7. Support renders as the contact list it is.
  out = out.replace(/<Support\b[\s\S]*?\/>/g, (tag) => {
    const techEmail = attr(tag, 'technicalSupportEmail');
    const techLabel = attr(tag, 'technicalSupportLabel') ?? 'Technical support';
    const coordinator = attr(tag, 'computeCoordinatorEmail') ?? URLS.coordinatorEmail;
    const rows = [];
    if (techEmail) rows.push(`- ${techLabel}: ${techEmail}`);
    rows.push(`- P1 compute coordinator: ${coordinator}`);
    rows.push(`- Book a consultation with the coordinator: ${URLS.booking}`);
    rows.push(
      `- Helped with your work? Acknowledge the coordinator's ORCID: ${URLS.coordinatorOrcid}`
    );
    return rows.join('\n');
  });

  // 8. Fixed wrapper components.
  for (const [name, render] of Object.entries(WRAPPERS)) {
    out = out.replace(new RegExp(`<${name}\\b[^>]*/>`, 'g'), () => render());
  }

  // 9. Anything left that looks like a component is presentational; drop it
  //    rather than emit raw Svelte into a file that claims to be markdown.
  out = out.replace(/<\/?[A-Z]\w*\b[^>]*>/g, '');

  // 10. Resolve the p1: link scheme the same way the remark plugin does.
  out = out.replace(/\]\(p1:(\w+)\)/g, (m, key: string) => {
    const url = URLS[key as keyof typeof URLS];
    return url ? `](${url})` : m;
  });

  // 11. Restore code, then tidy whitespace.
  out = out.replace(/%%FENCE(\d+)%%/g, (_m, i: string) => fences[Number(i)]);
  out = out.replace(/\n{3,}/g, '\n\n').trim();

  const docs = meta.providerDocs;
  const docsBlock = docs
    ? `> **Provider documentation.** This page is an overview to help you onboard and get started. ` +
      `The technical and advanced documentation is maintained by ${docs.provider}, who host and ` +
      `operate the system.\n>\n> ` +
      (docs.url
        ? `${docs.provider} documentation: ${docs.url}`
        : `${docs.provider} has not published technical documentation yet.`) +
      (docs.note ? `\n>\n> ${docs.note}` : '') +
      '\n\n'
    : '';

  const head = meta.title ? `# ${meta.title}\n\n${docsBlock}` : docsBlock;
  const tail = meta.hardware?.length
    ? `\n\n## Hardware Specification\n\n| | |\n| --- | --- |\n` +
      meta.hardware.map((h) => `| ${h.label} | ${h.value} |`).join('\n')
    : '';

  return `${head}${out}${tail}\n`;
}
