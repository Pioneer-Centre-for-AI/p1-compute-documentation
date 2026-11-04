import { z } from 'zod';
import { dev } from '$app/environment';
import { URLS } from '$lib/urls';

export const SESSION_TYPES = {
  int: 'Interactive',
  rem: 'Remote',
  bat: 'Batch',
  xfr: 'Transfer',
  srv: 'Service'
} as const;

export const GalleryFrontmatter = z.object({
  title: z.string(),
  description: z.string(),
  order: z.number().int(),
  navLabel: z.string(),
  summary: z.string(),
  tags: z.array(z.string().regex(/^[a-z0-9-]+$/)).min(1),
  clusters: z.array(z.enum(['dtu', 'gefion', 'ngc'])).min(1),
  thumbnail: z.object({ src: z.string(), alt: z.string() }).optional(),
  commands: z.array(z.string()).optional()
});

export type GalleryMeta = z.infer<typeof GalleryFrontmatter>;
export type GalleryEntry = {
  slug: string;
  code: string;
  typeLabel: string;
  draft: boolean;
  href: string;
  source: string | null;
  meta: GalleryMeta;
};

export const GALLERY_HREF = '/gallery/';

export const GALLERY_SOURCE = `${URLS.github}/tree/main/pkgs/gallery`;

export const GALLERY_INTRO = 'Example workflows for P1-affiliated researchers, and how to run them on P1 compute.';

const published = import.meta.glob('/src/routes/gallery/*/+page.svx', {
  eager: true
}) as Record<string, { metadata?: unknown }>;
const drafts = import.meta.glob('/src/routes/gallery/[(]drafts[)]/*/+page.svx', {
  eager: true
}) as Record<string, { metadata?: unknown }>;

function entry(path: string, mod: { metadata?: unknown }, draft: boolean): GalleryEntry {
  const slug = path.split('/').at(-2)!;
  const type = /^([a-z]{3})-[a-z]{3}$/.exec(slug)?.[1];
  if (!type || !(type in SESSION_TYPES)) {
    throw new Error(
      `Gallery page ${path}: "${slug}" is not a TYP-SCO code with a type from SESSION_TYPES ` +
        `(${Object.keys(SESSION_TYPES).join(', ')}).`
    );
  }
  return {
    slug,
    code: slug.toUpperCase(),
    typeLabel: SESSION_TYPES[type as keyof typeof SESSION_TYPES],
    draft,
    href: `${GALLERY_HREF}${slug}/`,
    source: __GALLERY_DIRS__.includes(slug) ? `${GALLERY_SOURCE}/${slug}` : null,
    meta: GalleryFrontmatter.parse(mod.metadata)
  };
}

var parsed: GalleryEntry[] | null | undefined;

export function getGallery(): GalleryEntry[] {
  parsed ??= [
    ...Object.entries(published).map(([path, mod]) => entry(path, mod, false)),
    ...(dev ? Object.entries(drafts).map(([path, mod]) => entry(path, mod, true)) : [])
  ].sort((a, b) => a.meta.order - b.meta.order);
  return parsed;
}
