<script lang="ts">
  import { page } from '$app/stores';
  import BrandAccent from '$lib/components/BrandAccent.svelte';
  import ClusterIllustration from '$lib/components/ClusterIllustration.svelte';
  import Meta from '$lib/components/Meta.svelte';
  import HardwareSpec from '$lib/components/HardwareSpec.svelte';
  import ProviderDocs from '$lib/components/ProviderDocs.svelte';
  import { headingAnchors } from '$lib/actions/headingAnchors';
  import { getClusters } from '$lib/content';

  let { children } = $props();

  // Null on /clusters/ itself, which is the overview page rather than a cluster.
  // It renders its own <article> and passes through the {:else} branch below;
  // only the detail pages get the header, illustration, and hardware table.
  const cluster = $derived(getClusters().find((c) => c.href === $page.url.pathname) ?? null);
</script>

{#if cluster}
  <Meta title={cluster.meta.title} description={cluster.meta.description} type="article" />
{/if}

{#if cluster}
  <!-- The TOC aside and the two-column shell come from the root layout. -->
  <article class="prose mx-auto w-full max-w-3xl" use:headingAnchors>
    <div class="not-prose flex items-end justify-between gap-6">
      <div>
        <div class="mb-4 flex items-center gap-3">
          <BrandAccent color={cluster.meta.brandColor} width="w-12" />
          <span class="eyebrow">Cluster · {cluster.meta.navLabel ?? cluster.meta.title}</span>
        </div>
        <h1 class="mb-0 mt-0 text-4xl font-extrabold tracking-tight text-slate-900 dark:text-slate-50">
          {cluster.meta.title}
        </h1>
      </div>
      <ClusterIllustration slug={cluster.slug} class="hidden w-36 shrink-0 sm:block md:w-44" />
    </div>
    <ProviderDocs {...cluster.meta.providerDocs} />
    {@render children()}
    <h2 id="hardware-specification">Hardware Specification</h2>
    <HardwareSpec items={cluster.meta.hardware} />
  </article>
{:else}
  <!-- /clusters/ overview: brings its own <article> and <svelte:head>. -->
  {@render children()}
{/if}
