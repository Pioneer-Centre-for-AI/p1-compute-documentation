import { z } from 'zod';
import { dev } from '$app/environment';
import { URLS } from '$lib/urls';

// Typed collection over the gallery pages, mirroring $lib/content for
// providers. Each /gallery/<code>/+page.svx carries its metadata in YAML
// frontmatter, validated here at build time. The code doubles as the directory
// name under pkgs/gallery/ at the repository root, which holds the example's
// files when it brings any.
//
// Pages under (drafts)/ are examples still in development. They are listed in
// `bun run dev` with a Draft badge and left out of production entirely: their
// own +layout.ts turns prerendering off and answers 404 outside dev.

/**
 * The first half of every code. The second half, the scope, is free: the
 * resource or subject, such as GPU, LGN for a login node, or DAT for data.
 */
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
  /** One sentence for the gallery cards. */
  summary: z.string(),
  /** Lowercase single words. They drive the gallery's filter chips and search. */
  tags: z.array(z.string().regex(/^[a-z0-9-]+$/)).min(1),
  /** Provider slugs the example runs on, as in /providers/<slug>/. */
  clusters: z.array(z.enum(['dtu', 'gefion', 'ngc'])).min(1),
  /**
   * A screenshot of the example running, kept in static/assets/gallery/.
   * Without one the card draws the example's line illustration instead.
   */
  thumbnail: z.object({ src: z.string(), alt: z.string() }).optional(),
  /**
   * What to run once the example's files are on the cluster, as
   * `aicentre gallery get <code>` prints them. Paths assume its default,
   * examples/<code> in the project directory.
   */
  commands: z.array(z.string()).optional()
});

export type GalleryMeta = z.infer<typeof GalleryFrontmatter>;
export type GalleryEntry = {
  slug: string;
  /** The slug in capitals, as cards and page eyebrows show it: INT-GPU. */
  code: string;
  /** The label for the code's session type: Interactive. */
  typeLabel: string;
  draft: boolean;
  href: string;
  /** The example's files in the repository, or null when it brings none. */
  source: string | null;
  meta: GalleryMeta;
};

export const GALLERY_HREF = '/gallery/';

/** Where the code lives in the site repository. */
export const GALLERY_SOURCE = `${URLS.github}/tree/main/pkgs/gallery`;

/** The subtitle under the gallery heading, shared by the page and its markdown twin. */
export const GALLERY_INTRO = 'Example workflows for P1-affiliated researchers, and how to run them on P1 compute.';

const published = import.meta.glob('/src/routes/gallery/*/+page.svx', {
  eager: true
}) as Record<string, { metadata?: unknown }>;
// Parentheses are glob syntax, hence the character classes around "drafts".
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

// Deferred exactly as getClusters() in $lib/content is, and for the same
// reason: the gallery layout imports this module, the glob imports the pages
// the layout wraps, and an eager parse reads frontmatter the cycle has not
// populated yet. See the note there for why this is `var`.
var parsed: GalleryEntry[] | null | undefined;

export function getGallery(): GalleryEntry[] {
  parsed ??= [
    ...Object.entries(published).map(([path, mod]) => entry(path, mod, false)),
    ...(dev ? Object.entries(drafts).map(([path, mod]) => entry(path, mod, true)) : [])
  ].sort((a, b) => a.meta.order - b.meta.order);
  return parsed;
}
