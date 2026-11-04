const raw = import.meta.glob('/pkgs/gallery/*/**', { query: '?raw', import: 'default', eager: true }) as Record<string, string>;

export function exampleFiles(slug: string): Record<string, string> {
  const prefix = `/pkgs/gallery/${slug}/`;
  const files: Record<string, string> = {};
  for (const [path, text] of Object.entries(raw)) if (path.startsWith(prefix)) files[path.slice(prefix.length)] = text;
  return files;
}

export function hasFiles(slug: string): boolean {
  return Object.keys(exampleFiles(slug)).length > 0;
}
