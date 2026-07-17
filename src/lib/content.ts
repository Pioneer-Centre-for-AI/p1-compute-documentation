import { z } from 'zod';

// Typed collection over the cluster pages. Each /clusters/<slug>/+page.svx
// carries its metadata (nav label, badge, hardware specs) in YAML frontmatter,
// validated here at build time.

const HardwareItem = z.object({ label: z.string(), value: z.string() });

export const ClusterFrontmatter = z.object({
  title: z.string(),
  description: z.string(),
  order: z.number().int(),
  navLabel: z.string().optional(),
  badge: z.object({ text: z.string(), variant: z.enum(['caution', 'note']) }).optional(),
  brandColor: z.string(),
  hardware: z.array(HardwareItem).min(1)
});

export type ClusterMeta = z.infer<typeof ClusterFrontmatter>;
export type ClusterEntry = { slug: string; href: string; meta: ClusterMeta };

const modules = import.meta.glob('/src/routes/clusters/*/+page.svx', {
  eager: true
}) as Record<string, { metadata?: unknown }>;

export const clusters: ClusterEntry[] = Object.entries(modules)
  .map(([path, mod]) => {
    const slug = path.split('/').at(-2)!;
    return { slug, href: `/clusters/${slug}/`, meta: ClusterFrontmatter.parse(mod.metadata) };
  })
  .sort((a, b) => a.meta.order - b.meta.order);

export function getCluster(slug: string): ClusterEntry | undefined {
  return clusters.find((c) => c.slug === slug);
}
