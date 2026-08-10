import { z } from 'zod';

// Typed collection over the event pages, mirroring $lib/content for clusters.
// Each /events/<slug>/+page.svx carries its metadata in YAML frontmatter,
// validated here at build time.

export const EventFrontmatter = z.object({
  title: z.string(),
  description: z.string(),
  order: z.number().int(),
  navLabel: z.string().optional(),
  /** Context line above the title, e.g. the conference the session belongs to. */
  eyebrow: z.string(),
  /** ISO date. Drives the schema.org startDate. */
  startDate: z.string(),
  /** Human-readable date shown in the page body. */
  dateLabel: z.string(),
  /** Compact form for the site-wide notice bar on narrow screens. */
  shortLabel: z.string().optional(),
  /** ISO 8601 duration for schema.org, e.g. PT90M. */
  duration: z.string().optional(),
  venue: z.string().optional()
});

export type EventMeta = z.infer<typeof EventFrontmatter>;
export type EventEntry = { slug: string; href: string; meta: EventMeta };

const modules = import.meta.glob('/src/routes/events/*/+page.svx', {
  eager: true
}) as Record<string, { metadata?: unknown }>;

export const events: EventEntry[] = Object.entries(modules)
  .map(([path, mod]) => {
    const slug = path.split('/').at(-2)!;
    return { slug, href: `/events/${slug}/`, meta: EventFrontmatter.parse(mod.metadata) };
  })
  .sort((a, b) => a.meta.order - b.meta.order);

/**
 * Soonest event that has not happened yet, or null. ISO dates compare
 * correctly as strings, so no Date parsing is needed.
 */
export function upcomingEvent(today = new Date().toISOString().slice(0, 10)): EventEntry | null {
  return (
    events
      .filter((e) => e.meta.startDate >= today)
      .sort((a, b) => a.meta.startDate.localeCompare(b.meta.startDate))[0] ?? null
  );
}

// --- Supercomputing: Agents & Challenges (D3A 4.0 2026) ---------------------
//
// The programme, organizers, and collaboration rows live here rather than in
// the page, so the HTML and the .md twin render the same content from one
// source. Same reasoning as clusterDecisions in $lib/content.

/** The three at-a-glance panels above the programme. */
export const superAgentAtAGlance: { label: string; items: string[] }[] = [
  {
    label: 'Format',
    items: ['90 minutes, interactive', 'Talks 33% or less', 'Discussion 61% or more']
  },
  {
    label: 'Level',
    items: ['Intermediate', 'HPC experience helpful', 'No agent expertise required']
  },
  {
    label: 'Audience',
    items: ['AI/ML researchers', 'HPC operators', 'Domain scientists on shared compute']
  }
];

/** Row kind, which picks the stripe colour and the legend entry. */
export type SessionType = 'framing' | 'talk' | 'interactive';

export type ProgrammeRow = {
  time: string;
  session: string;
  speaker: string;
  speakerUrl?: string;
  /** Institution shown after the speaker name. */
  affiliation?: string;
  type: SessionType;
};

export const sessionTypes: { type: SessionType; label: string; color: string }[] = [
  { type: 'framing', label: 'Framing and synthesis', color: 'var(--color-slate-400)' },
  { type: 'talk', label: 'Talk', color: 'var(--color-plum)' },
  { type: 'interactive', label: 'Interactive', color: 'var(--color-coral)' }
];

export const sessionTypeColor: Record<SessionType, string> = Object.fromEntries(
  sessionTypes.map((t) => [t.type, t.color])
) as Record<SessionType, string>;

// Times are offsets from the start of the 90-minute slot. Swap to absolute
// clock times once the conference fixes the slot.
export const superAgentProgramme: ProgrammeRow[] = [
  {
    time: '00:00-00:05',
    session: 'Welcome and framing',
    speaker: 'Benjamin Starostka Jakobsen',
    speakerUrl: 'https://starostka.io/',
    type: 'framing'
  },
  {
    time: '00:05-00:15',
    session: 'Agentic HPC: GenAI and sustainability',
    speaker: 'Claudio Pica',
    speakerUrl: 'https://portal.findresearcher.sdu.dk/en/persons/pica/',
    affiliation: 'SDU eScience Center',
    type: 'talk'
  },
  {
    time: '00:15-00:25',
    session: 'Operating HPC: Hosting research at scale',
    speaker: 'Ulrik N. de Lichtenberg',
    speakerUrl: 'https://www.linkedin.com/in/ulrik-nicolai-de-lichtenberg',
    affiliation: 'DCAI / Gefion',
    type: 'talk'
  },
  {
    time: '00:25-00:35',
    session: 'Shared HPC: Effective sharing of GPUs',
    speaker: 'Pınar Tözün',
    speakerUrl: 'https://www.pinartozun.com/',
    affiliation: 'IT University of Copenhagen',
    type: 'talk'
  },
  {
    time: '00:35-00:55',
    session: 'Breakout discussions: HPC providers and researchers',
    speaker: 'All participants',
    type: 'interactive'
  },
  {
    time: '00:55-01:15',
    session: 'Plenum: report-back from breakouts',
    speaker: 'Benjamin Starostka Jakobsen',
    speakerUrl: 'https://starostka.io/',
    type: 'interactive'
  },
  {
    time: '01:15-01:30',
    session: 'Synthesis: common findings and next steps',
    speaker: 'Nicki Skafte Detlefsen',
    speakerUrl: 'https://www.linkedin.com/in/nicki-skafte-detlefsen',
    type: 'framing'
  }
];

/** Shaped to the Person type that People.svelte expects. */
export const superAgentOrganizers = [
  {
    name: 'Nicki Skafte Detlefsen',
    role: 'DTU',
    institution: 'Technical University of Denmark',
    email: 'nsde@dtu.dk',
    profileUrl: 'https://www.linkedin.com/in/nicki-skafte-detlefsen'
  },
  {
    name: 'Ulrik N. de Lichtenberg',
    role: 'DCAI',
    institution: 'Danish Centre for AI Innovation',
    email: 'ulrik.de.lichtenberg@dcai.dk',
    profileUrl: 'https://www.linkedin.com/in/ulrik-nicolai-de-lichtenberg'
  },
  {
    name: 'Benjamin Starostka Jakobsen',
    role: 'P1',
    institution: 'Pioneer Centre for Artificial Intelligence',
    email: 'bstja@dtu.dk',
    profileUrl: 'https://starostka.io/'
  }
];

export type Collaboration = { name: string; href: string; logo: string; blurb: string };

export const superAgentCollaborations: Collaboration[] = [
  {
    name: 'Pioneer Centre for Artificial Intelligence',
    href: 'https://www.aicentre.dk/',
    logo: '/assets/logos/PCAI_Horizontal.svg',
    blurb: 'Pan-Danish research centre advancing AI, machine learning, and computer vision.'
  },
  {
    name: 'Technical University of Denmark',
    href: 'https://www.hpc.dtu.dk/',
    logo: '/assets/logos/DTU.svg',
    blurb:
      'DTU Computing Center provides centralised HPC services, software stacks, storage, and training to researchers across the university.'
  },
  {
    name: 'Danish Centre for AI Innovation',
    href: 'https://dcai.dk/gefion',
    logo: '/assets/logos/DCAI.svg',
    blurb:
      "Home to Gefion, Denmark's NVIDIA DGX SuperPOD (191 DGX H100 nodes, 1,528 H100 GPUs) for large-scale AI projects."
  }
];
