import * as links from '$lib/links';
import { QUOTA_GROUPS, type ClusterQuotaMeta } from '$lib/content';
import { URLS } from '$lib/urls';

type Meta = {
  title?: string;
  hardware?: { label: string; value: string }[];
  providerDocs?: { provider: string; url?: string; note?: string };
  quotas?: ClusterQuotaMeta;
  blocks?: Record<string, string>;
};

const REPLACE_REGION = /<!--\s*md:replace\s+([\w-]+)\s*-->[\s\S]*?<!--\s*md:end\s*-->/g;

function cell(value: string): string {
  return value.replace(/\|/g, '\\|');
}

export function markdownTable(headers: string[], rows: string[][]): string {
  const head = `| ${headers.map(cell).join(' | ')} |`;
  const rule = `| ${headers.map(() => '---').join(' | ')} |`;
  const body = rows.map((r) => `| ${r.map(cell).join(' | ')} |`);
  return [head, rule, ...body].join('\n');
}

const FENCE = /^```[\s\S]*?^```/gm;

function resolveAttr(raw: string | undefined): string | undefined {
  if (!raw) return undefined;
  const expr = /^\{([^{}]*)\}$/.exec(raw.trim())?.[1]?.trim() ?? raw.trim();
  const known = (links as Record<string, unknown>)[expr];
  if (typeof known === 'string') return known;
  const interpolated = expr.replace(/\{(\w+)\}/g, (m, name) => {
    const value = (links as Record<string, unknown>)[name];
    return typeof value === 'string' ? value : m;
  });
  return interpolated || undefined;
}

function attr(tag: string, name: string): string | undefined {
  const quoted = new RegExp(`\\b${name}="([^"]*)"`).exec(tag)?.[1];
  if (quoted !== undefined) return quoted;
  return new RegExp(`\\b${name}=(\\{[^}]*\\})`).exec(tag)?.[1];
}

function formLine(title: string, url: string | undefined, description?: string): string {
  const target = url ?? '';
  const suffix = description ? ` ${description}` : '';
  return `**Form:** [${title}](${target})${suffix}`;
}

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

  out = out.replace(/^---\r?\n[\s\S]*?\r?\n---\r?\n/, '');

  const fences: string[] = [];
  out = out.replace(FENCE, (block) => {
    fences.push(block);
    return `%%FENCE${fences.length - 1}%%`;
  });

  out = out.replace(REPLACE_REGION, (_m, name: string) => {
    const block = meta.blocks?.[name];
    if (block === undefined) {
      throw new Error(`Page uses <!-- md:replace ${name} --> but no block was supplied for it.`);
    }
    return block;
  });

  out = out.replace(/<script[\s\S]*?<\/script>/g, '');
  out = out.replace(/<svelte:head[\s\S]*?<\/svelte:head>/g, '');
  out = out.replace(/<\/?article\b[^>]*>/g, '');
  out = out.replace(/<p class="eyebrow[^"]*"[^>]*>[\s\S]*?<\/p>/g, '');
  out = out.replace(/<img\b[^>]*>/g, '');

  out = out.replace(/<Aside\b([^>]*)>([\s\S]*?)<\/Aside>/g, (_m, rawAttrs: string, body: string) => {
    const label = attr(rawAttrs, 'title') ?? attr(rawAttrs, 'type') ?? 'Note';
    const quoted = body
      .trim()
      .split('\n')
      .map((line) => (line.trim() ? `> ${line}` : '>'))
      .join('\n');
    return `> **${label}**\n>\n${quoted}`;
  });

  out = out.replace(/<EmbedForm\b[\s\S]*?\/>/g, (tag) => {
    const title = attr(tag, 'title') ?? 'Form';
    const url = resolveAttr(attr(tag, 'href')) ?? resolveAttr(attr(tag, 'src'));
    return formLine(title, url, attr(tag, 'description'));
  });

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

  for (const [name, render] of Object.entries(WRAPPERS)) {
    out = out.replace(new RegExp(`<${name}\\b[^>]*/>`, 'g'), () => render());
  }

  out = out.replace(/<\/?[A-Z]\w*\b[^>]*>/g, '');

  out = out.replace(/\]\(p1:(\w+)\)/g, (m, key: string) => {
    const url = URLS[key as keyof typeof URLS];
    return url ? `](${url})` : m;
  });

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
