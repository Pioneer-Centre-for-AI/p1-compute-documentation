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

  const links: NavItem[] = nav.flatMap((entry) => (isGroup(entry) ? entry.items : [entry]));
  const year = new Date().getFullYear();
</script>

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

    <nav aria-label="Footer">
      <div class="eyebrow mb-3">Site</div>
      <ul class="grid grid-cols-2 gap-x-6 gap-y-1.5 text-sm">
        {#each links as link}
          <li>
            <a
              href={link.href}
              class="text-slate-600 no-underline hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-100"
            >
              {link.label}
            </a>
          </li>
        {/each}
      </ul>
    </nav>

    <div>
      <div class="eyebrow mb-3">Contact</div>
      <ul class="space-y-1.5 text-sm">
        <!-- Labelled rather than spelling the addresses out: two bare addresses
             give no clue which to use, and the target shows on hover anyway. -->
        <li>
          <a
            href="mailto:{COORDINATOR_EMAIL}"
            title={COORDINATOR_EMAIL}
            class="text-slate-600 no-underline hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-100"
          >
            Compute coordinator
          </a>
        </li>
        <li>
          <a
            href="mailto:{GOVERNANCE_EMAIL}"
            title={GOVERNANCE_EMAIL}
            class="text-slate-600 no-underline hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-100"
          >
            Governance and policy
          </a>
        </li>
        <li>
          <a
            href={SURVEY_PAGE}
            class="text-slate-600 no-underline hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-100"
          >
            Tell us about your workload
          </a>
        </li>
        <li>
          <!-- Points at the index rather than /about/, so it is equally useful
               pasted into an agent and clicked by a curious human. -->
          <a
            href="/llms.txt"
            title="Index of this site for AI agents, per llmstxt.org"
            class="text-slate-600 no-underline hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-100"
          >
            Docs for AI agents
          </a>
        </li>
        <li>
          <a
            href={GITHUB_URL}
            target="_blank"
            rel="noopener noreferrer"
            class="text-slate-600 no-underline hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-100"
          >
            GitHub ↗
          </a>
        </li>
        <li>
          <a
            href="https://aicentre.dk"
            target="_blank"
            rel="noopener noreferrer"
            class="text-slate-600 no-underline hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-100"
          >
            aicentre.dk ↗
          </a>
        </li>
        <li>
          <a
            href={PROFILE_URL}
            target="_blank"
            rel="noopener noreferrer"
            class="text-slate-600 no-underline hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-100"
          >
            Update your P1 profile ↗
          </a>
        </li>
      </ul>
    </div>
  </div>

  <div class="border-t border-slate-200 dark:border-slate-800">
    <div class="eyebrow mx-auto flex w-full max-w-7xl flex-wrap items-center gap-x-6 gap-y-2 px-4 py-4 md:px-6">
      <span>© {year} Pioneer Centre for AI</span>
      <a href="/terms/" class="no-underline hover:text-slate-700 dark:hover:text-slate-200">Terms</a>
      <a href="/privacy/" class="no-underline hover:text-slate-700 dark:hover:text-slate-200">Privacy</a>
    </div>
  </div>
</footer>
