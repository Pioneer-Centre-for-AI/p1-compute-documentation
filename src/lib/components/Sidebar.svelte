<script lang="ts">
  import { page } from '$app/stores';
  import { nav, isGroup, type NavItem } from '$lib/nav';

  function isActive(href: string, current: string): boolean {
    if (href.startsWith('http')) return false;
    const norm = (s: string) => (s.endsWith('/') ? s : s + '/');
    return norm(current) === norm(href);
  }
</script>

<nav class="text-sm" aria-label="Primary">
  <ul class="space-y-6">
    {#each nav as entry}
      {#if isGroup(entry)}
        <li>
          <div class="mb-2 px-2 text-xs font-semibold uppercase tracking-wider text-slate-500 dark:text-slate-400">
            {entry.label}
          </div>
          <ul class="space-y-0.5">
            {#each entry.items as item}
              {@const active = isActive(item.href, $page.url.pathname)}
              <li>
                <a
                  href={item.href}
                  class="flex items-center justify-between rounded-md px-2 py-1.5 no-underline {active ? 'bg-blue-50 text-blue-700 dark:bg-blue-900/30 dark:text-blue-300' : 'text-slate-700 hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-slate-800'}"
                >
                  <span>{item.label}</span>
                  {#if item.badge}
                    <span class="ml-2 rounded bg-amber-100 px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-amber-800 dark:bg-amber-900/40 dark:text-amber-200">
                      {item.badge.text}
                    </span>
                  {/if}
                </a>
              </li>
            {/each}
          </ul>
        </li>
      {:else}
        {@const item = entry as NavItem}
        {@const active = isActive(item.href, $page.url.pathname)}
        <li>
          <a
            href={item.href}
            target={item.newTab ? '_blank' : undefined}
            rel={item.newTab ? 'noopener noreferrer' : undefined}
            class="flex items-center justify-between rounded-md px-2 py-1.5 no-underline {active ? 'bg-blue-50 text-blue-700 dark:bg-blue-900/30 dark:text-blue-300' : 'text-slate-700 hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-slate-800'}"
          >
            <span>{item.label}</span>
            {#if item.newTab}
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M15 3h6v6M10 14L21 3M21 14v7H3V3h7" />
              </svg>
            {/if}
          </a>
        </li>
      {/if}
    {/each}
  </ul>
</nav>
