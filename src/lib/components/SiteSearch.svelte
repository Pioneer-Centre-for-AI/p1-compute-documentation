<script lang="ts">
  import { goto } from '$app/navigation';
  import { onMount, tick } from 'svelte';
  import { launcher } from '#lib/search.svelte.ts';
  import type { SearchEntry } from '#lib/markdown/search.ts';

  const KIND_LABELS: Record<SearchEntry['kind'], string> = {
    page: 'Page',
    section: 'Section',
    requirement: 'Requirement'
  };

  let dialog = $state<HTMLDialogElement | null>(null);
  let input = $state<HTMLInputElement | null>(null);
  let entries = $state<SearchEntry[] | null>(null);
  let query = $state('');
  let active = $state(0);
  let shortcut = $state('Ctrl K');

  const results = $derived(rank(entries ?? [], query));

  function rank(all: SearchEntry[], q: string): SearchEntry[] {
    const terms = q.toLowerCase().split(/\s+/).filter(Boolean);
    if (terms.length === 0) return all.filter((e) => e.kind === 'page').slice(0, 8);
    const scored: { entry: SearchEntry; matched: number; score: number }[] = [];
    for (const entry of all) {
      const title = entry.title.toLowerCase();
      const section = (entry.section ?? '').toLowerCase();
      const body = entry.body.toLowerCase();
      let matched = 0;
      let score = entry.kind === 'page' ? 1 : 0;
      for (const t of terms) {
        const hit = title.includes(t) || section.includes(t) || body.includes(t);
        if (!hit) continue;
        matched += 1;
        if (title === t) score += 12;
        else if (title.startsWith(t)) score += 6;
        else if (title.includes(t)) score += 4;
        if (section.includes(t)) score += 3;
        if (body.includes(t)) score += 1;
      }
      if (matched) scored.push({ entry, matched, score });
    }
    return scored
      .sort((a, b) => b.matched - a.matched || b.score - a.score)
      .slice(0, 20)
      .map((s) => s.entry);
  }

  async function load() {
    if (entries) return;
    const response = await fetch('/search.json');
    entries = response.ok ? await response.json() : [];
  }

  function open(entry: SearchEntry | undefined) {
    if (!entry) return;
    launcher.open = false;
    goto(entry.href);
  }

  function onInputKey(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      active = results.length ? (active + 1) % results.length : 0;
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      active = results.length ? (active - 1 + results.length) % results.length : 0;
    } else if (e.key === 'Enter') {
      e.preventDefault();
      open(results[active]);
    }
  }

  function typing(target: EventTarget | null): boolean {
    if (!(target instanceof HTMLElement)) return false;
    return target.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName);
  }

  $effect(() => {
    query;
    active = 0;
  });

  $effect(() => {
    const el = dialog;
    if (!el) return;
    if (launcher.open) {
      if (!el.open) {
        query = '';
        el.showModal();
        load();
        tick().then(() => input?.focus());
      }
      return;
    }
    if (el.open) el.close();
  });

  $effect(() => {
    document.getElementById(`site-search-option-${active}`)?.scrollIntoView({ block: 'nearest' });
  });

  onMount(() => {
    if (/Mac|iPhone|iPad/.test(navigator.platform)) shortcut = '⌘K';
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        launcher.open = !launcher.open;
      } else if (e.key === '/' && !launcher.open && !typing(e.target)) {
        e.preventDefault();
        launcher.open = true;
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  });
</script>

<dialog
  bind:this={dialog}
  aria-label="Search the site"
  onclose={() => (launcher.open = false)}
  onclick={(e) => {
    if (e.target === dialog) launcher.open = false;
  }}
  class="mx-auto mt-[12vh] mb-auto max-h-none max-w-none overflow-visible border-0 bg-transparent p-0 backdrop:bg-slate-950/50 backdrop:backdrop-blur-sm"
>
  <div
    class="flex max-h-[70vh] w-[min(40rem,100vw-2rem)] flex-col overflow-hidden rounded-xl border border-slate-200 bg-white shadow-2xl dark:border-slate-700 dark:bg-slate-900"
  >
    <div class="flex items-center gap-3 border-b border-slate-200 px-4 dark:border-slate-700">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true" class="shrink-0 text-slate-500">
        <circle cx="11" cy="11" r="7" />
        <line x1="16.5" y1="16.5" x2="21" y2="21" />
      </svg>
      <input
        bind:this={input}
        bind:value={query}
        onkeydown={onInputKey}
        type="text"
        role="combobox"
        aria-expanded="true"
        aria-controls="site-search-results"
        aria-activedescendant={results.length ? `site-search-option-${active}` : undefined}
        aria-autocomplete="list"
        placeholder="Search pages, sections and RFC requirements"
        class="h-12 min-w-0 flex-1 bg-transparent text-base text-slate-900 outline-none placeholder:text-slate-500 dark:text-slate-100 dark:placeholder:text-slate-400"
      />
      <kbd class="hidden rounded border border-slate-200 px-1.5 py-0.5 font-mono text-[10px] text-slate-500 sm:inline dark:border-slate-700 dark:text-slate-400">Esc</kbd>
    </div>

    <ul id="site-search-results" role="listbox" aria-label="Results" class="min-h-0 flex-1 overflow-y-auto p-2">
      {#if entries === null}
        <li class="px-3 py-6 text-center text-sm text-slate-500 dark:text-slate-400">Loading…</li>
      {:else if results.length === 0}
        <li class="px-3 py-6 text-center text-sm text-slate-500 dark:text-slate-400">No results for “{query}”.</li>
      {:else}
        {#each results as entry, i (entry.href + entry.title + (entry.section ?? '') + i)}
          <li
            id="site-search-option-{i}"
            role="option"
            aria-selected={i === active}
            onmousemove={() => (active = i)}
            class="rounded-lg {i === active ? 'bg-slate-100 dark:bg-slate-800' : ''}"
          >
            <a
              href={entry.href}
              tabindex="-1"
              onclick={(e) => {
                e.preventDefault();
                open(entry);
              }}
              class="flex items-start justify-between gap-4 px-3 py-2 no-underline"
            >
              <span class="min-w-0">
                <span class="block truncate text-sm font-semibold text-slate-900 dark:text-slate-100">
                  {entry.kind === 'section' ? entry.section : entry.title}
                </span>
                <span class="block truncate text-xs text-slate-600 dark:text-slate-400">
                  {entry.kind === 'page' ? entry.text : entry.kind === 'section' ? entry.title : `${entry.section}: ${entry.text}`}
                </span>
              </span>
              <span class="shrink-0 pt-0.5 font-mono text-[10px] uppercase tracking-wide text-slate-500 dark:text-slate-400">
                {KIND_LABELS[entry.kind]}
              </span>
            </a>
          </li>
        {/each}
      {/if}
    </ul>

    <div class="flex items-center justify-between gap-4 border-t border-slate-200 px-4 py-2 text-xs text-slate-500 dark:border-slate-700 dark:text-slate-400">
      <span>↑ ↓ to move · Enter to open · Esc to close</span>
      <span class="font-mono">{shortcut}</span>
    </div>
  </div>
</dialog>
