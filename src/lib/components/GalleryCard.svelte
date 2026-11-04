<script lang="ts">
  import Button from './Button.svelte';
  import ClusterDots from './ClusterDots.svelte';
  import GalleryIllustration from './GalleryIllustration.svelte';
  import type { GalleryEntry } from '$lib/gallery';

  type Props = { entry: GalleryEntry; onTag?: (tag: string) => void };
  let { entry, onTag }: Props = $props();
</script>

<div
  class="group flex flex-col rounded-xl border border-slate-200 bg-slate-50/60 p-4 transition-all hover:-translate-y-0.5 hover:shadow-lg dark:border-slate-700 dark:bg-slate-800/40"
>
  <a
    href={entry.href}
    tabindex="-1"
    aria-hidden="true"
    class="block aspect-[4/3] overflow-hidden rounded-lg border border-slate-200 bg-white dark:border-slate-700 dark:bg-slate-900"
  >
    {#if entry.meta.thumbnail}
      <img
        src={entry.meta.thumbnail.src}
        alt={entry.meta.thumbnail.alt}
        loading="lazy"
        class="h-full w-full object-cover object-top"
      />
    {:else}
      <GalleryIllustration slug={entry.slug} class="flex h-full items-center px-6" />
    {/if}
  </a>

  <div class="flex flex-1 flex-col gap-2 px-1 pt-4">
    <span class="flex items-center gap-2">
      <span class="eyebrow">{entry.code} · {entry.typeLabel}</span>
      {#if entry.draft}
        <span class="rounded bg-[var(--color-highlight)] px-1.5 py-0.5 font-mono text-[10px] font-medium uppercase tracking-wide text-slate-900">
          Draft
        </span>
      {/if}
    </span>
    <a
      href={entry.href}
      class="text-lg font-semibold leading-snug text-slate-900 no-underline hover:underline dark:text-slate-100"
    >
      {entry.meta.title}
    </a>
    <ClusterDots slugs={entry.meta.clusters} />
    <p class="line-clamp-3 flex-1 text-sm leading-relaxed text-slate-500 dark:text-slate-400">
      {entry.meta.summary}
    </p>

    <ul class="mt-1 flex flex-wrap gap-1.5">
      {#each entry.meta.tags as tag}
        <li>
          {#if onTag}
            <button
              type="button"
              onclick={() => onTag(tag)}
              class="rounded-full bg-slate-200/70 px-2.5 py-0.5 text-xs text-slate-600 transition-colors hover:bg-slate-300/70 dark:bg-slate-700/70 dark:text-slate-300 dark:hover:bg-slate-600/70"
            >
              {tag}
            </button>
          {:else}
            <span
              class="inline-block rounded-full bg-slate-200/70 px-2.5 py-0.5 text-xs text-slate-600 dark:bg-slate-700/70 dark:text-slate-300"
            >
              {tag}
            </span>
          {/if}
        </li>
      {/each}
    </ul>

    <div class="mt-3 flex flex-wrap items-center gap-2">
      <Button href={entry.href} size="sm">How it works</Button>
      {#if entry.source}
        <Button href={entry.source} variant="ghost" size="sm" target="_blank" rel="noopener noreferrer">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <polyline points="16 18 22 12 16 6" />
            <polyline points="8 6 2 12 8 18" />
          </svg>
          Source
        </Button>
      {/if}
    </div>
  </div>
</div>
