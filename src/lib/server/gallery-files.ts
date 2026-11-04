// The files of each example in pkgs/gallery/, read at build time, for the
// tarballs at /gallery/<code>.tar.gz that `aicentre gallery get` unpacks on
// a cluster, so nobody has to clone the repository to try one.
const raw = import.meta.glob('/pkgs/gallery/*/**', { query: '?raw', import: 'default', eager: true }) as Record<string, string>;

/** path inside the example -> text, for one example. */
export function exampleFiles(slug: string): Record<string, string> {
  const prefix = `/pkgs/gallery/${slug}/`;
  const files: Record<string, string> = {};
  for (const [path, text] of Object.entries(raw)) if (path.startsWith(prefix)) files[path.slice(prefix.length)] = text;
  return files;
}

/** Whether the example brings any files to fetch. */
export function hasFiles(slug: string): boolean {
  return Object.keys(exampleFiles(slug)).length > 0;
}
