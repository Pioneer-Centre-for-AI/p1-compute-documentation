<script lang="ts">
  // The clusters an example runs on, each with its brand-colour dot. Resolved
  // while rendering rather than at import, for the cycle noted in $lib/nav.
  // With `href`, each cluster links somewhere, such as the gallery filtered to it.
  import { getCluster } from '$lib/content';

  type Props = { slugs: string[]; href?: (slug: string) => string; class?: string };
  let { slugs, href, class: className = '' }: Props = $props();

  const clusters = $derived(slugs.map((s) => getCluster(s)).filter((c) => c !== undefined));
</script>

<span class="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-slate-500 dark:text-slate-400 {className}">
  {#each clusters as cluster}
    {#snippet label()}
      <span
        class="inline-block h-2 w-2 shrink-0 rounded-full"
        style:background-color={cluster.meta.brandColor}
        aria-hidden="true"
      ></span>
      {cluster.meta.navLabel ?? cluster.meta.title}
    {/snippet}
    {#if href}
      <a
        href={href(cluster.slug)}
        class="inline-flex items-center gap-1.5 text-slate-500 no-underline hover:text-[var(--color-brand)] dark:text-slate-400"
      >
        {@render label()}
      </a>
    {:else}
      <span class="inline-flex items-center gap-1.5">{@render label()}</span>
    {/if}
  {/each}
</span>
