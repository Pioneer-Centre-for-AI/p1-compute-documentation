<script lang="ts">
  import { SITE_URL } from '$lib/site';

  type Props = { class?: string };
  let { class: className = '' }: Props = $props();

  const command = `curl -fsSL ${SITE_URL}/install | sh`;
  let copied = $state(false);

  async function copy() {
    try {
      await navigator.clipboard.writeText(command);
      copied = true;
      setTimeout(() => (copied = false), 1500);
    } catch {}
  }
</script>

<div class="flex w-full max-w-xl flex-col gap-2 {className}">
  <div
    class="flex items-center gap-3 rounded-lg border border-slate-200 bg-slate-50 py-1.5 pl-4 pr-1.5 font-mono text-sm dark:border-slate-700 dark:bg-slate-900"
  >
    <span class="select-none text-slate-400 dark:text-slate-500" aria-hidden="true">$</span>
    <code class="min-w-0 flex-1 overflow-x-auto whitespace-nowrap text-slate-800 dark:text-slate-100">{command}</code>
    <button
      type="button"
      onclick={copy}
      aria-label="Copy the install command"
      class="inline-flex shrink-0 items-center gap-1.5 rounded-md px-2.5 py-1.5 font-sans text-xs font-semibold text-slate-600 transition-colors hover:bg-slate-200 hover:text-slate-900 dark:text-slate-300 dark:hover:bg-slate-700 dark:hover:text-white"
    >
      {#if copied}
        <span>✓ Copied</span>
      {:else}
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
          <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
        </svg>
        <span>Copy</span>
      {/if}
    </button>
    <span class="sr-only" aria-live="polite">{copied ? 'Install command copied' : ''}</span>
  </div>
  <p class="text-xs text-slate-500 dark:text-slate-400">
    <span class="mr-1 rounded bg-amber-100 px-1.5 py-0.5 font-semibold text-amber-900 dark:bg-amber-900/40 dark:text-amber-200">Work in progress</span>
    A command line tool for P1 compute, illustrative for now.
    <a href="/cli/" class="font-medium text-slate-600 underline decoration-slate-300 underline-offset-2 hover:text-[var(--color-brand)] dark:text-slate-300 dark:decoration-slate-600">What it does</a>
  </p>
</div>
