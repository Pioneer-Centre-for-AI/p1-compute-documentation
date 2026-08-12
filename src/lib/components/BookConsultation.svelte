<script lang="ts">
  import BookingDialog from './BookingDialog.svelte';
  import { BOOKING_URL } from '$lib/links';

  type Props = {
    title?: string;
    description?: string;
  };
  let {
    title = 'Book a consultation',
    description = 'In person or online, with the compute coordinator'
  }: Props = $props();

  let open = $state(false);
</script>

<div
  class="not-prose my-4 flex flex-wrap items-center gap-x-3 gap-y-2 rounded-lg border border-slate-200 bg-slate-50 px-3 py-2.5 dark:border-slate-700 dark:bg-slate-800/40"
>
  <div class="min-w-0 flex-1">
    <span class="text-sm font-semibold text-slate-900 dark:text-slate-100">{title}</span>
    {#if description}
      <span class="text-xs text-slate-500 dark:text-slate-400"> · {description}</span>
    {/if}
  </div>
  <div class="flex shrink-0 items-center gap-2">
    <button
      type="button"
      onclick={() => (open = true)}
      class="inline-flex h-8 items-center rounded-full bg-slate-800 px-4 text-sm font-medium text-white transition-colors hover:bg-slate-700 dark:bg-slate-200 dark:text-slate-900 dark:hover:bg-slate-300"
    >
      Pick a time
    </button>
    <a
      href={BOOKING_URL}
      target="_blank"
      rel="noopener noreferrer"
      class="inline-flex h-8 items-center gap-1.5 rounded-full border border-slate-300 px-3 text-sm font-medium text-slate-700 no-underline transition-colors hover:bg-slate-100 dark:border-slate-600 dark:text-slate-200 dark:hover:bg-slate-800"
    >
      New tab
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M15 3h6v6M10 14L21 3M21 14v7H3V3h7" />
      </svg>
    </a>
  </div>
</div>

<BookingDialog bind:open {title} />
