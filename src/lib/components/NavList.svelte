<script lang="ts">
  import { page } from '$app/stores';
  import { nav, isGroup, type NavItem } from '$lib/nav';

  function isActive(href: string, current: string): boolean {
    if (href.startsWith('http')) return false;
    const norm = (s: string) => (s.endsWith('/') ? s : s + '/');
    return norm(current) === norm(href);
  }
</script>

{#snippet navLink(item: NavItem)}
  {@const active = isActive(item.href, $page.url.pathname)}
  <a
    href={item.href}
    target={item.newTab ? '_blank' : undefined}
    rel={item.newTab ? 'noopener noreferrer' : undefined}
    aria-current={active ? 'page' : undefined}
    class="flex items-center justify-between rounded-md px-2 py-1.5 no-underline {active ? 'bg-slate-100 font-semibold text-slate-900 dark:bg-slate-800 dark:text-slate-50' : 'text-slate-600 hover:bg-slate-50 hover:text-slate-900 dark:text-slate-400 dark:hover:bg-slate-800/60 dark:hover:text-slate-100'}"
  >
    <span class="flex items-center gap-2">
      {#if item.dot}
        <span class="h-1.5 w-1.5 shrink-0 rounded-full" style:background-color={item.dot} aria-hidden="true"></span>
      {/if}
      {item.label}
    </span>
    {#if item.badge}
      <span class="ml-2 rounded bg-slate-200 px-1.5 py-0.5 font-mono text-[10px] font-medium uppercase tracking-wide text-slate-700 dark:bg-slate-700 dark:text-slate-200">
        {item.badge.text}
      </span>
    {:else if item.newTab}
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M15 3h6v6M10 14L21 3M21 14v7H3V3h7" />
      </svg>
    {/if}
  </a>
{/snippet}

<nav class="text-sm" aria-label="Primary">
  <ul class="space-y-7">
    {#each nav as entry}
      {#if isGroup(entry)}
        <li>
          <div class="eyebrow mb-2 px-2">
            {entry.label}
          </div>
          <ul class="space-y-0.5 border-l border-slate-200 pl-1 dark:border-slate-800">
            {#each entry.items as item}
              <li>{@render navLink(item)}</li>
            {/each}
          </ul>
        </li>
      {:else}
        <li>{@render navLink(entry as NavItem)}</li>
      {/if}
    {/each}
  </ul>
</nav>
