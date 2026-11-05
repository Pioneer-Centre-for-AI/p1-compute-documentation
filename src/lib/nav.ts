import { getClusters } from '#lib/content.ts';
import { getEvents } from '#lib/events.ts';

export type NavBadgeVariant = 'caution' | 'note';

export type NavItem = {
  label: string;
  href: string;
  badge?: { text: string; variant: NavBadgeVariant };
  dot?: string;
  external?: boolean;
  newTab?: boolean;
};

export type NavGroup = {
  label: string;
  items: NavItem[];
};

export type NavEntry = NavItem | NavGroup;

export function getNav(): NavEntry[] {
  return [
    {
      label: 'Getting Started',
      items: [
        { label: 'About', href: '/about/' },
        { label: 'Your Compute Needs', href: '/survey/' },
        { label: 'Command Line Tool', href: '/cli/', badge: { text: 'WIP', variant: 'caution' } },
        { label: 'Forms & Bookings', href: '/forms/' }
      ]
    },
    {
      label: 'Providers',
      items: [
        { label: 'Overview', href: '/providers/' },
        ...getClusters().map((c) => ({
          label: c.meta.navLabel ?? c.meta.title,
          href: c.href,
          badge: c.meta.badge,
          dot: c.meta.brandColor
        }))
      ]
    },
    {
      label: 'Resources',
      items: [
        { label: 'Ecosystem', href: '/ecosystem/' },
        { label: 'Learn', href: '/learn/' }
      ]
    },
    {
      label: 'Events',
      items: [
        { label: 'Overview', href: '/events/' },
        ...getEvents().map((e) => ({ label: e.meta.navLabel ?? e.meta.title, href: e.href }))
      ]
    }
  ];
}

export function isGroup(entry: NavEntry): entry is NavGroup {
  return 'items' in entry;
}
