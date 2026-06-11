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
      { label: 'DTU HPC', href: '/clusters/dtu-hpc/' },
      {
        label: 'NGC HPC',
        href: '/clusters/ngc-hpc/',
        badge: { text: 'WAITING LIST', variant: 'caution' }
      },
      {
        label: 'DCAI Gefion',
        href: '/clusters/dcai-gefion/',
        badge: { text: 'INTEREST LIST', variant: 'caution' }
      }
    ]
  },
  {
    label: 'Reference',
    items: [{ label: 'Learning Resources', href: '/reference/learning-resources/' }]
  },
  { label: 'Terms and Conditions', href: '/terms-and-conditions/' },
  {
    label: 'Computing',
    href: 'https://www.aicentre.dk/computing',
    external: true,
    newTab: true
  }
];

export function isGroup(entry: NavEntry): entry is NavGroup {
  return 'items' in entry;
}
