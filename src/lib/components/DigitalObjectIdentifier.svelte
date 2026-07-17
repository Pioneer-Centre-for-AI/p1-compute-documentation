<script lang="ts">
  type Props = { doi?: string };
  let { doi = 'https://doi.org/10.48714/DTU.HPC.0001' }: Props = $props();
  const displayDoi = $derived(doi.replace('https://doi.org/', ''));
  let copied = $state(false);

  async function copy() {
    try {
      await navigator.clipboard.writeText(doi);
      copied = true;
      setTimeout(() => (copied = false), 1500);
    } catch {}
  }
</script>

<button
  type="button"
  onclick={copy}
  aria-label="Copy DOI to clipboard"
  class="inline-flex items-center justify-center gap-2 rounded-md bg-slate-800 px-4 py-2 text-sm font-semibold text-white transition-colors hover:bg-slate-700 dark:bg-slate-700 dark:hover:bg-slate-600"
>
  {#if copied}
    <span>✓ Copied</span>
  {:else}
    <span class="rounded bg-white/25 px-2 py-0.5 text-xs font-bold tracking-wide text-white">DOI</span>
    <span class="font-mono text-white">{displayDoi}</span>
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-white opacity-80" aria-hidden="true">
      <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
      <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
    </svg>
  {/if}
</button>
