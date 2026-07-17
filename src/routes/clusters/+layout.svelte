<script lang="ts">
  import { page } from '$app/stores';
  import BrandAccent from '$lib/components/BrandAccent.svelte';
  import ClusterPageToc from '$lib/components/ClusterPageToc.svelte';
  import HardwareSpec from '$lib/components/HardwareSpec.svelte';
  import { headingAnchors } from '$lib/actions/headingAnchors';
  import { clusters } from '$lib/content';

  let { children } = $props();

  const cluster = $derived(clusters.find((c) => c.href === $page.url.pathname) ?? null);
</script>

<svelte:head>
  {#if cluster}
    <title>{cluster.meta.title}</title>
    <meta name="description" content={cluster.meta.description} />
  {/if}
</svelte:head>

{#if cluster}
  <div class="mx-auto flex w-full max-w-5xl gap-12">
    <article class="prose min-w-0 flex-1 max-w-3xl" use:headingAnchors>
      <BrandAccent color={cluster.meta.brandColor} />
      <h1>{cluster.meta.title}</h1>
      {@render children()}
      <h2 id="hardware-specification">Hardware Specification</h2>
      <HardwareSpec items={cluster.meta.hardware} />
    </article>
    <aside class="hidden w-48 shrink-0 lg:block">
      <div class="sticky top-20">
        <ClusterPageToc />
      </div>
    </aside>
  </div>
{:else}
  {@render children()}
{/if}
