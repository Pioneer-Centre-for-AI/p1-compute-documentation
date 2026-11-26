import { z } from 'zod';

const HardwareItem = z.object({ label: z.string(), value: z.string() });

const ProviderDocs = z.object({
  provider: z.string(),
  url: z.string().optional(),
  note: z.string().optional()
});

export type ProviderDocsMeta = z.infer<typeof ProviderDocs>;

const Quotas = z.object({
  wallTime: z.string(),
  gpusPerJob: z.string(),
  gpusAtOnce: z.string(),
  homeDirectory: z.string(),
  projectStorage: z.string(),
  sharedPool: z.string()
});

export type ClusterQuotaMeta = z.infer<typeof Quotas>;

export const QUOTA_GROUPS: {
  title: string;
  rows: { key: keyof ClusterQuotaMeta; label: string }[];
}[] = [
  {
    title: 'Compute',
    rows: [
      { key: 'wallTime', label: 'Max wall time' },
      { key: 'gpusPerJob', label: 'GPUs per job' },
      { key: 'gpusAtOnce', label: 'GPUs at once, per person' }
    ]
  },
  {
    title: 'Storage',
    rows: [
      { key: 'homeDirectory', label: 'Home directory' },
      { key: 'projectStorage', label: 'Project storage' },
      { key: 'sharedPool', label: 'Shared pool' }
    ]
  }
];

export const ClusterFrontmatter = z.object({
  title: z.string(),
  description: z.string(),
  order: z.number().int(),
  navLabel: z.string().optional(),
  badge: z.object({ text: z.string(), variant: z.enum(['caution', 'note']) }).optional(),
  brandColor: z.string(),
  providerDocs: ProviderDocs,
  quotas: Quotas,
  hardware: z.array(HardwareItem).min(1)
});

export type ClusterMeta = z.infer<typeof ClusterFrontmatter>;
export type ClusterEntry = { slug: string; href: string; meta: ClusterMeta };

const modules = import.meta.glob('/src/routes/providers/*/+page.svx', {
  eager: true
}) as Record<string, { metadata?: unknown }>;

var parsed: ClusterEntry[] | null | undefined;

export function getClusters(): ClusterEntry[] {
  parsed ??= Object.entries(modules)
    .map(([path, mod]) => {
      const slug = path.split('/').at(-2)!;
      return { slug, href: `/providers/${slug}/`, meta: ClusterFrontmatter.parse(mod.metadata) };
    })
    .sort((a, b) => a.meta.order - b.meta.order);
  return parsed;
}

export function getCluster(slug: string): ClusterEntry | undefined {
  return getClusters().find((c) => c.slug === slug);
}

export type ClusterDecision = { need: string; slug: string; why: string };

export const clusterDecisions: ClusterDecision[] = [
  {
    need: 'General ML/AI research on non-sensitive data',
    slug: 'dtu',
    why: 'Free compute and storage, batch + interactive H100 access'
  },
  {
    need: 'Personal data in a strictly controlled setting',
    slug: 'ngc',
    why: 'Air-gapped, GDPR-compliant environment'
  },
  {
    need: 'To mature a workload for large-scale runs',
    slug: 'gefion',
    why: 'DGX H100 sandbox in Gefion AI Cloud, for non-sensitive data; the step to full nodes needs its own funding'
  }
];

export type ComputeTier = { tier: string; level: string; description: string };

export const computeTiers: ComputeTier[] = [
  {
    tier: '0',
    level: 'European',
    description:
      'The supercomputers of the EuroHPC Joint Undertaking, for the largest projects. Projects usually start with 8-16 GPUs and can scale to hundreds, and to more than a thousand on the largest systems. Limits are set on total GPU hours rather than wall time. Danish researchers reach them through DeiC and EuroHPC calls. See the <a href="https://eurohpc-ju.europa.eu/supercomputers/our-supercomputers_en">EuroHPC supercomputers</a>, such as LUMI, Karolina and MeluXina.'
  },
  {
    tier: '1',
    level: 'National',
    description:
      'National HPC facilities provide the resources for large-scale model training. Expect fewer restrictions to wall time (1-2 weeks), storage (around 1-10 TB) and number of nodes/GPUs used (around 16-64). In Denmark these are Gefion, where P1 holds a sandbox partition, and the <a href="https://www.deic.dk/en/supercomputing-hpc/computing-power-through-deic/national-facilities">DeiC national HPC systems</a>.'
  },
  {
    tier: '2',
    level: 'Regional',
    description:
      'Regional systems, open to researchers across a region. Denmark has no separate regional level: clusters shared across Danish universities are national (Tier 1).'
  },
  {
    tier: '3',
    level: 'Institutional',
    description:
      'Compute clusters containing either several smaller GPUs or a few larger ones. Multiple experiments can be run in parallel and models can be parallelized over a few GPUs, but it is not feasible to train very large models from scratch. Expect limitations to wall time (a couple of days), storage (around 0.5-1 TB) and number of nodes/GPUs used (around 2-16).'
  },
  {
    tier: '–',
    level: 'Personal',
    description:
      'Personal laptop and/or desktop with either no GPUs or a single one, outside the European scale. At this level compute is limited and mostly consists of code development and running experiments with small datasets and models.'
  }
];

export type ClusterDecisionEntry = ClusterDecision & { cluster: ClusterEntry };

export function getClusterDecisionEntries(): ClusterDecisionEntry[] {
  return clusterDecisions.map((d) => {
    const cluster = getCluster(d.slug);
    if (!cluster) throw new Error(`Decision row references unknown cluster "${d.slug}"`);
    return { ...d, cluster };
  });
}
