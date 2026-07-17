import { clusters } from '$lib/content';

export type NavBadgeVariant = 'caution' | 'note';

export type NavItem = {
  label: string;
  href: string;
  badge?: { text: string; variant: NavBadgeVariant };
  external?: boolean;
  newTab?: boolean;
};

export type NavGroup = {
  label: string;
  items: NavItem[];
};

export type NavEntry = NavItem | NavGroup;

export const nav: NavEntry[] = [
  { label: 'About', href: '/about/' },
  {
    label: 'Clusters',
    items: [
      { label: 'Overview', href: '/clusters/' },
      ...clusters.map((c) => ({
        label: c.meta.navLabel ?? c.meta.title,
        href: c.href,
        badge: c.meta.badge
      }))
    ]
  },
  { label: 'Ecosystem', href: '/ecosystem/' },
  { label: 'Learn', href: '/learn/' },
  { label: 'Terms', href: '/terms/' },
  { label: 'Privacy', href: '/privacy/' }
];

export function isGroup(entry: NavEntry): entry is NavGroup {
  return 'items' in entry;
}
