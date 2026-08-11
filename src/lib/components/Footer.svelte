<script lang="ts">
  import { nav, isGroup, type NavItem } from '$lib/nav';
  import {
    SURVEY_PAGE,
    PROFILE_URL,
    GITHUB_URL,
    COORDINATOR_EMAIL,
    GOVERNANCE_EMAIL
  } from '$lib/links';
  import DigitalObjectIdentifier from './DigitalObjectIdentifier.svelte';

  type FooterLink = {
    label: string;
    href: string;
    /** Shown on hover, for links whose target is not obvious from the label. */
    title?: string;
    external?: boolean;
  };

  // Grouped rather than flattened: two nav groups have an "Overview" item, and
  // side by side in one list they read as duplicates of each other.
  const siteGroups = nav.filter(isGroup);
  const looseLinks: NavItem[] = nav.filter((entry) => !isGroup(entry)) as NavItem[];

  // Ways to reach a person. Emails are labelled rather than spelled out: two
  // bare addresses give no clue which to use, and the target shows on hover.
  const contact: FooterLink[] = [
    { label: 'Compute coordinator', href: `mailto:${COORDINATOR_EMAIL}`, title: COORDINATOR_EMAIL },
    { label: 'Governance and policy', href: `mailto:${GOVERNANCE_EMAIL}`, title: GOVERNANCE_EMAIL },
    { label: 'Tell us about your workload', href: SURVEY_PAGE }
  ];

  // P1 properties that are not this site.
  const centre: FooterLink[] = [
    { label: 'aicentre.dk', href: 'https://aicentre.dk', external: true },
    { label: 'Update your P1 profile', href: PROFILE_URL, external: true }
  ];

  // Meta links about the site itself rather than about P1 compute.
  const meta: FooterLink[] = [
    { label: 'Terms', href: '/terms/' },
    { label: 'Privacy', href: '/privacy/' },
    {
      label: 'Docs for AI agents',
      href: '/llms.txt',
      title: 'Index of this site for AI agents, per llmstxt.org'
    },
    { label: 'Source on GitHub', href: GITHUB_URL, external: true }
  ];

  const linkClass =
    'text-slate-600 no-underline hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-100';

  const year = new Date().getFullYear();
</script>

{#snippet linkGroup(label: string, items: FooterLink[])}
  <div>
    <div class="eyebrow mb-3">{label}</div>
    <ul class="space-y-1.5 text-sm">
      {#each items as item}
        <li>
          <a
            href={item.href}
            title={item.title}
            target={item.external ? '_blank' : undefined}
            rel={item.external ? 'noopener noreferrer' : undefined}
            class={linkClass}
          >
            {item.label}{item.external ? ' ↗' : ''}
          </a>
        </li>
      {/each}
    </ul>
  </div>
{/snippet}

<footer class="border-t border-slate-200 bg-slate-50 dark:border-slate-800 dark:bg-slate-950">
  <div class="mx-auto grid w-full max-w-7xl gap-10 px-4 py-12 sm:grid-cols-2 md:px-6 lg:grid-cols-3">
    <div class="flex flex-col gap-3">
      <a href="/" class="flex items-center gap-3 no-underline">
        <img src="/assets/logos/PCAI_Solo_RGB.svg" alt="" class="block h-8 w-8 dark:hidden" />
        <img src="/assets/logos/PCAI_Solo_White.svg" alt="" class="hidden h-8 w-8 dark:block" />
        <span class="text-sm font-semibold text-slate-900 dark:text-slate-100">
          Pioneer Centre for AI
        </span>
      </a>
      <p class="max-w-xs text-sm leading-relaxed text-slate-500 dark:text-slate-400">
        We fund and help run partitions at partner HPC facilities, train and guide researchers, and
        point the way across the wider compute landscape.
      </p>
      <div class="mt-2">
        <p class="mb-2 text-xs text-slate-500 dark:text-slate-400">
          Did the P1 DTU HPC help your work? Please cite:
        </p>
        <DigitalObjectIdentifier />
      </div>
    </div>

    <!-- Two columns of groups when there is room. At `sm` this cell is only
         half the viewport, which is too narrow for two, so it stacks. -->
    <nav aria-label="Footer" class="grid grid-cols-2 gap-x-6 gap-y-8 sm:grid-cols-1 lg:grid-cols-2">
      {#each siteGroups as group}
        {@render linkGroup(group.label, group.items)}
      {/each}
      {#if looseLinks.length}
        {@render linkGroup('More', looseLinks)}
      {/if}
    </nav>

    <div class="flex flex-col gap-8">
      {@render linkGroup('Contact', contact)}
      {@render linkGroup('Pioneer Centre', centre)}
    </div>
  </div>

  <div class="border-t border-slate-200 dark:border-slate-800">
    <div class="eyebrow mx-auto flex w-full max-w-7xl flex-wrap items-center gap-x-6 gap-y-2 px-4 py-4 md:px-6">
      <span>© {year} Pioneer Centre for AI</span>
      {#each meta as item}
        <a
          href={item.href}
          title={item.title}
          target={item.external ? '_blank' : undefined}
          rel={item.external ? 'noopener noreferrer' : undefined}
          class="no-underline hover:text-slate-700 dark:hover:text-slate-200"
        >
          {item.label}
        </a>
      {/each}
    </div>
  </div>
</footer>
