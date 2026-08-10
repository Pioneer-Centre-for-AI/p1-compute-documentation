<script lang="ts">
  import { page } from '$app/stores';
  import Button from '$lib/components/Button.svelte';

  const isMissing = $derived($page.status === 404);
</script>

<svelte:head>
  <title>{isMissing ? 'Page not found' : 'Something went wrong'}</title>
  <meta name="robots" content="noindex" />
</svelte:head>

<article class="prose mx-auto w-full max-w-3xl py-12">
  <p class="eyebrow not-prose mb-4">Error {$page.status}</p>

  <h1>{isMissing ? 'Page not found' : 'Something went wrong'}</h1>

  {#if isMissing}
    <p>
      That page does not exist, or it moved. The cluster pages were reorganised recently, so an old
      bookmark is the likely cause.
    </p>
  {:else}
    <p>{$page.error?.message ?? 'An unexpected error occurred.'}</p>
  {/if}

  <div class="not-prose mt-8 flex flex-wrap gap-3">
    <Button href="/clusters/" variant="primary">
      {#snippet children()}
        Browse the clusters
      {/snippet}
    </Button>
    <Button href="/" variant="ghost">
      {#snippet children()}
        Back to the front page
      {/snippet}
    </Button>
  </div>
</article>
