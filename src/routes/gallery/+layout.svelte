<script lang="ts">
  import { page } from '$app/state';
  import BrandAccent from '#lib/components/BrandAccent.svelte';
  import ClusterDots from '#lib/components/ClusterDots.svelte';
  import Meta from '#lib/components/Meta.svelte';
  import { headingAnchors } from '#lib/actions/headingAnchors.ts';
  import { GALLERY_HREF, getGallery } from '#lib/gallery.ts';

  let { children } = $props();

  const example = $derived(getGallery().find((e) => e.href === page.url.pathname) ?? null);
</script>

{#if example}
  <Meta title={example.meta.title} description={example.meta.description} type="article" />
{/if}

{#if example}
  <article class="prose mx-auto w-full max-w-3xl" use:headingAnchors>
    <div class="not-prose">
      <a
        href={GALLERY_HREF}
        class="mb-6 inline-flex text-sm font-medium text-slate-500 no-underline hover:text-[var(--color-brand)] dark:text-slate-400"
      >
        ← Gallery
      </a>
      <div class="mb-4 flex items-center gap-3">
        <BrandAccent color="var(--color-brand)" width="w-12" />
        <span class="eyebrow">{example.code} · {example.typeLabel}</span>
      </div>
      <h1
        class="mb-0 mt-0 text-4xl font-extrabold tracking-tight text-slate-900 dark:text-slate-50"
      >
        {example.meta.title}
      </h1>
      <div class="mt-4 flex flex-wrap items-center gap-x-4 gap-y-2">
        <ClusterDots slugs={example.meta.clusters} href={(slug) => `${GALLERY_HREF}?provider=${slug}`} />
        <ul class="flex flex-wrap gap-1.5">
          {#each example.meta.tags as tag}
            <li>
              <a
                href="{GALLERY_HREF}?tag={tag}"
                class="inline-block rounded-full bg-slate-100 px-2.5 py-0.5 text-xs text-slate-600 no-underline hover:bg-slate-200 dark:bg-slate-800 dark:text-slate-300 dark:hover:bg-slate-700"
              >
                {tag}
              </a>
            </li>
          {/each}
        </ul>
      </div>
    </div>
    {@render children()}
  </article>
{:else}
  <div class="mx-auto w-full max-w-7xl px-4 py-10 md:px-6 md:py-16">
    {@render children()}
  </div>
{/if}
