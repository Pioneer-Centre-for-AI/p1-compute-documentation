import { getClusters } from '$lib/content';
import { getEvents } from '$lib/events';

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

// A function, not a const: the cluster entries come from $lib/content, which
// sits in an import cycle with the cluster pages it globs. Building this list
// at module evaluation reached into that cycle before it had settled and took
// every page down with a 500. Callers ask for it while rendering, by which
// point the cycle is resolved.
//
// Terms and Privacy intentionally live only in the footer.
export function getNav(): NavEntry[] {
  return [
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
