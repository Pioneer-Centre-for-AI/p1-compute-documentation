<script lang="ts">
  import type { ProviderDocsMeta } from '$lib/content';

  let { provider, url, note }: ProviderDocsMeta = $props();
</script>

<aside
  class="not-prose my-6 rounded-xl border border-slate-200 bg-slate-50 p-5 dark:border-slate-700 dark:bg-slate-800/40"
>
  <p class="eyebrow mb-2">Provider documentation</p>
  <p class="text-sm leading-relaxed text-slate-600 dark:text-slate-400">
    This page is an overview to help you onboard and get started. The technical and advanced
    documentation is maintained by <span class="font-semibold text-slate-700 dark:text-slate-300"
      >{provider}</span
    >, who host and operate the system.
  </p>
  {#if url}
    <a
      href={url}
      target="_blank"
      rel="noopener noreferrer"
      class="mt-3 inline-flex items-center gap-2 text-sm font-semibold text-[var(--color-brand)] no-underline hover:underline"
    >
      {provider} documentation
      <svg
        width="14"
        height="14"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
        <polyline points="15 3 21 3 21 9" />
        <line x1="10" y1="14" x2="21" y2="3" />
      </svg>
    </a>
  {:else}
    <p class="mt-3 text-sm font-semibold text-slate-700 dark:text-slate-300">
      {provider} has not published technical documentation yet.
    </p>
  {/if}
  {#if note}
    <p class="mt-2 text-xs leading-relaxed text-slate-500 dark:text-slate-400">{note}</p>
  {/if}
</aside>
