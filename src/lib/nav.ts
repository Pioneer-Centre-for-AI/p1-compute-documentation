import { clusters } from '$lib/content';
import { events } from '$lib/events';

export type NavBadgeVariant = 'caution' | 'note';

export type NavItem = {
  label: string;
  href: string;
  badge?: { text: string; variant: NavBadgeVariant };
  /** Small colored dot in front of the label, e.g. a cluster brand color. */
  dot?: string;
  external?: boolean;
  newTab?: boolean;
};

export type NavGroup = {
  label: string;
  items: NavItem[];
};

export type NavEntry = NavItem | NavGroup;

// Terms and Privacy intentionally live only in the footer.
export const nav: NavEntry[] = [
  {
    label: 'Getting Started',
    items: [
      { label: 'About', href: '/about/' },
      { label: 'Your Compute Needs', href: '/survey/' },
      // The route stays /forms/: renaming it would move the nav entry, the
      // llms.txt link, the sitemap URL and anything anyone has bookmarked, to
      // change a word in a label.
      { label: 'Forms & Bookings', href: '/forms/' }
    ]
  },
  {
    label: 'Clusters',
    items: [
      { label: 'Overview', href: '/clusters/' },
      ...clusters.map((c) => ({
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
      ...events.map((e) => ({ label: e.meta.navLabel ?? e.meta.title, href: e.href }))
    ]
  }
];

export function isGroup(entry: NavEntry): entry is NavGroup {
  return 'items' in entry;
}
