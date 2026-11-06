<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import GalleryCard from './GalleryCard.svelte';
  import { getClusters } from '#lib/content.ts';
  import type { GalleryEntry } from '#lib/gallery.ts';

  type Props = { entries: GalleryEntry[] };
  let { entries }: Props = $props();

  let query = $state('');
  let cluster = $state<string | null>(null);
  let tag = $state<string | null>(null);

  const runsOn = (e: GalleryEntry, slug: string) => (e.meta.clusters as string[]).includes(slug);

  const clusters = $derived(
    getClusters().map((c) => ({
      slug: c.slug,
      label: c.meta.title,
      color: c.meta.brandColor,
      count: entries.filter((e) => runsOn(e, c.slug)).length
    }))
  );
  const tags = $derived([...new Set(entries.flatMap((e) => e.meta.tags))].sort());

  const haystacks = $derived(
    new Map(
      entries.map((e) => [
        e.slug,
        [e.code, e.typeLabel, e.meta.title, e.meta.navLabel, e.meta.summary, e.meta.description, ...e.meta.tags]
          .join(' ')
          .toLowerCase()
      ])
    )
  );
  const tokens = $derived(query.toLowerCase().split(/\s+/).filter(Boolean));
  const shown = $derived(
    entries.filter(
      (e) =>
        (cluster === null || runsOn(e, cluster)) &&
        (tag === null || e.meta.tags.includes(tag)) &&
        tokens.every((t) => haystacks.get(e.slug)!.includes(t))
    )
  );

  onMount(() => {
    const params = new URLSearchParams(location.search);
    const c = params.get('provider');
    const t = params.get('tag');
    if (c && clusters.some((x) => x.slug === c && x.count > 0)) cluster = c;
    if (t && tags.includes(t)) tag = t;
  });

  function sync() {
    const url = new URL(location.href);
    for (const [key, value] of [
      ['provider', cluster],
      ['tag', tag]
    ] as const) {
      if (value) url.searchParams.set(key, value);
      else url.searchParams.delete(key);
    }
    goto(url, { state: {}, shallow: true, replace: true });
  }

  function chooseCluster(next: string | null) {
    cluster = next;
    sync();
  }

  function chooseTag(next: string | null) {
    tag = next;
    sync();
  }

  function clear() {
    query = '';
    cluster = null;
    tag = null;
    sync();
  }

  const chip = 'rounded-full px-3 py-1 text-sm transition-colors';
  const chipOff =
    'bg-slate-100 text-slate-600 hover:bg-slate-200 dark:bg-slate-800 dark:text-slate-300 dark:hover:bg-slate-700';
  const chipOn = 'bg-[var(--color-brand)] text-white dark:text-slate-900';

  const pill =
    'inline-flex items-center gap-2 rounded-full border px-3 py-1 text-sm transition-colors disabled:cursor-not-allowed disabled:opacity-40';
  const pillOff =
    'border-slate-300 text-slate-600 enabled:hover:bg-slate-100 dark:border-slate-600 dark:text-slate-300 dark:enabled:hover:bg-slate-800';
  const pillOn =
    'border-[var(--color-brand)] bg-[var(--color-brand)]/10 text-slate-900 dark:text-slate-50';
</script>

<div class="flex flex-col gap-3 sm:flex-row sm:items-center">
  <label class="relative flex-1">
    <span class="sr-only">Search the gallery</span>
    <svg
      class="pointer-events-none absolute top-1/2 left-3.5 -translate-y-1/2 text-slate-400"
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      aria-hidden="true"
    >
      <circle cx="11" cy="11" r="7" />
      <line x1="16.5" y1="16.5" x2="21" y2="21" />
    </svg>
    <input
      type="search"
      bind:value={query}
      placeholder="Search examples…"
      class="h-11 w-full rounded-lg border border-slate-300 bg-white pr-4 pl-10 text-base text-slate-900 placeholder:text-slate-500 sm:text-sm focus:border-[var(--color-brand)] focus:ring-2 focus:ring-[var(--color-brand)]/20 focus:outline-none dark:border-slate-600 dark:bg-slate-900 dark:text-slate-100"
    />
  </label>
  <p class="shrink-0 text-sm text-slate-500 dark:text-slate-400" aria-live="polite">
    {shown.length}
    {shown.length === 1 ? 'example' : 'examples'}
  </p>
</div>

<div class="mt-5 flex flex-wrap items-center gap-2" role="group" aria-label="Filter by provider">
  <span class="eyebrow mr-1">Runs on</span>
  <button type="button" class="{pill} {cluster === null ? pillOn : pillOff}" aria-pressed={cluster === null} onclick={() => chooseCluster(null)}>
    Any provider
  </button>
  {#each clusters as c}
    <button
      type="button"
      class="{pill} {cluster === c.slug ? pillOn : pillOff}"
      aria-pressed={cluster === c.slug}
      disabled={c.count === 0}
      title={c.count === 0 ? 'No examples yet' : undefined}
      onclick={() => chooseCluster(c.slug)}
    >
      <span class="inline-block h-2 w-2 shrink-0 rounded-full" style:background-color={c.color} aria-hidden="true"></span>
      {c.label}
      <span class="text-xs text-slate-500 dark:text-slate-400">{c.count}</span>
    </button>
  {/each}
</div>

<div class="mt-3 flex flex-wrap gap-2" role="group" aria-label="Filter by tag">
  <button type="button" class="{chip} {tag === null ? chipOn : chipOff}" aria-pressed={tag === null} onclick={() => chooseTag(null)}>
    All
  </button>
  {#each tags as t}
    <button type="button" class="{chip} {tag === t ? chipOn : chipOff}" aria-pressed={tag === t} onclick={() => chooseTag(t)}>
      {t}
    </button>
  {/each}
</div>

{#if shown.length > 0}
  <div class="mt-8 grid gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
    {#each shown as entry (entry.slug)}
      <GalleryCard {entry} onTag={chooseTag} />
    {/each}
  </div>
{:else}
  <div
    class="mt-8 flex flex-col items-center gap-3 rounded-xl border border-dashed border-slate-300 px-6 py-14 text-center dark:border-slate-600"
  >
    <p class="text-sm text-slate-500 dark:text-slate-400">No example matches.</p>
    <button
      type="button"
      onclick={clear}
      class="text-sm font-semibold text-[var(--color-brand)] hover:underline"
    >
      Clear filters
    </button>
  </div>
{/if}
