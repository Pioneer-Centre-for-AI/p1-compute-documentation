<script lang="ts">
  import { page } from '$app/state';
  import Meta from '#lib/components/Meta.svelte';
  import PageToc from '#lib/components/PageToc.svelte';
  import RfcStatus from '#lib/components/RfcStatus.svelte';
  import { headingAnchors } from '#lib/actions/headingAnchors.ts';
  import { markdownHref } from '#lib/markdownPages.ts';
  import { STATUS_LABELS, formatDay, getRfcs, rfcTitle } from '#lib/rfcs.ts';
  import { SITE_URL } from '#lib/site.ts';

  let { children } = $props();

  const rfc = $derived(getRfcs().find((r) => r.href === page.url.pathname) ?? null);
  const address = $derived(rfc ? `${SITE_URL.replace(/^https:\/\//, '')}${rfc.href}` : '');

  const dt = 'text-slate-600 dark:text-slate-400';
  const dd = 'text-slate-900 dark:text-slate-100';
  const link = 'font-semibold text-[var(--color-brand)] no-underline hover:underline';
</script>

<svelte:head>
  <link rel="alternate" type="application/atom+xml" title="RFCs" href="/rfc/feed.xml" />
</svelte:head>

{#if rfc}
  <Meta title={rfcTitle(rfc)} description={rfc.meta.description} type="article" />

  <div data-rfc-document class="mx-auto w-full max-w-6xl px-4 py-8 md:px-6 md:py-12">
    <nav aria-label="Breadcrumb" data-print-hide class="mb-10 flex items-center justify-between gap-4 text-sm">
      <a href="/rfc/" class="text-slate-600 no-underline hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-100">
        <span aria-hidden="true">←</span> Requests for comments
      </a>
      <a href="/rfc/all/" class="font-mono text-xs text-slate-600 no-underline hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-100">
        All RFCs
      </a>
    </nav>

    <header class="grid gap-x-10 gap-y-3 md:grid-cols-[9rem_minmax(0,1fr)]">
      <p class="font-mono text-sm text-slate-600 uppercase md:pt-1 dark:text-slate-400">RFC {rfc.number}</p>
      <div>
        <div class="flex flex-wrap items-center gap-2">
          <RfcStatus status={rfc.meta.status} />
          <span
            class="rounded bg-amber-100 px-1.5 py-0.5 font-semibold text-amber-900 dark:bg-amber-900/40 dark:text-amber-200 text-xs"
            title="These RFCs show how the process could work. Their requirements may change, and no provider has committed to them."
            >Concept</span
          >
        </div>
        <h1
          class="mt-3 text-4xl font-extrabold tracking-tight hyphens-auto text-slate-900 [overflow-wrap:break-word] md:text-5xl dark:text-slate-50"
        >
          {rfc.meta.title}
        </h1>
        <p class="mt-4 max-w-[65ch] text-lg leading-relaxed text-slate-600 dark:text-slate-400">
          {rfc.meta.description}
        </p>
      </div>
    </header>

    <dl
      class="mt-8 grid grid-cols-[6.5rem_minmax(0,1fr)] gap-x-4 gap-y-2 border-y border-slate-200 py-5 text-sm md:grid-cols-[9rem_minmax(0,1fr)] md:gap-x-10 dark:border-slate-800"
    >
      <dt class={dt}>{rfc.meta.authors.length > 1 ? 'Authors' : 'Author'}</dt>
      <dd class={dd}>{rfc.meta.authors.join(', ')}</dd>
      <dt class={dt}>Created</dt>
      <dd class={dd}>{formatDay(rfc.meta.created)}</dd>
      <dt class={dt}>Updated</dt>
      <dd class={dd}>{formatDay(rfc.meta.updated)}</dd>
      {#if rfc.meta.decided}
        <dt class={dt}>{STATUS_LABELS[rfc.meta.status]}</dt>
        <dd class={dd}>{formatDay(rfc.meta.decided)}</dd>
      {/if}
      {#if rfc.meta.summary}
        <dt class={dt}>Decision</dt>
        <dd class={dd}>{rfc.meta.summary}</dd>
      {/if}
      {#if rfc.meta.status === 'open'}
        <dt class={dt}>Comments</dt>
        <dd class={dd}>Any channel, such as the <a href="/survey/" class={link}>compute needs survey</a></dd>
      {/if}
      <dt class={dt}>Address</dt>
      <dd class="font-mono text-xs break-all text-slate-900 md:pt-0.5 dark:text-slate-100">{address}</dd>
    </dl>

    <div class="mt-10 grid gap-12 lg:grid-cols-[minmax(0,1fr)_13rem]">
      <article class="prose min-w-0 max-w-3xl" use:headingAnchors>
        {@render children()}
      </article>
      <aside data-print-hide class="hidden lg:block">
        <div class="sticky top-24">
          <PageToc mdHref={markdownHref(page.url.pathname)} />
        </div>
      </aside>
    </div>
  </div>
{:else}
  {@render children()}
{/if}
