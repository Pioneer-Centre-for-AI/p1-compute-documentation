<script lang="ts">
  import { BOOKING_URL } from '$lib/links';

  let { open = $bindable(false), title = 'Book a consultation' }: { open?: boolean; title?: string } =
    $props();

  let dialog = $state<HTMLDialogElement | null>(null);
  let src = $state('');

  $effect(() => {
    const el = dialog;
    if (!el) return;
    if (open) {
      if (!el.open) {
        const theme = document.documentElement.classList.contains('dark') ? 'dark' : 'light';
        src = `${BOOKING_URL}?embed=true&layout=month_view&theme=${theme}`;
        el.showModal();
      }
      const prev = document.body.style.overflow;
      document.body.style.overflow = 'hidden';
      return () => {
        document.body.style.overflow = prev;
      };
    }
    if (el.open) el.close();
  });
</script>

<dialog
  bind:this={dialog}
  aria-label={title}
  onclose={() => (open = false)}
  onclick={(e) => {
    // A click on the backdrop targets the dialog itself; anything inside the
    // panel targets a descendant.
    if (e.target === dialog) open = false;
  }}
  class="m-auto max-h-none max-w-none overflow-visible border-0 bg-transparent p-0 backdrop:bg-slate-950/50 backdrop:backdrop-blur-sm"
>
  <div
    class="flex h-[min(85vh,46rem)] w-[min(72rem,100vw-2rem)] flex-col overflow-hidden rounded-xl bg-white shadow-2xl dark:bg-slate-900"
  >
    <div class="flex items-center justify-between gap-3 border-b border-slate-200 px-4 py-3 dark:border-slate-700">
      <p class="min-w-0 truncate text-sm font-semibold text-slate-900 dark:text-slate-100">{title}</p>
      <div class="flex shrink-0 items-center gap-2">
        <a
          href={BOOKING_URL}
          target="_blank"
          rel="noopener noreferrer"
          class="inline-flex h-8 items-center gap-1.5 rounded-md border border-slate-200 px-3 text-xs font-medium text-slate-600 no-underline hover:bg-slate-100 dark:border-slate-700 dark:text-slate-300 dark:hover:bg-slate-800"
        >
          Open in new tab
          <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M15 3h6v6M10 14L21 3M21 14v7H3V3h7" />
          </svg>
        </a>
        <button
          aria-label="Close booking"
          onclick={() => (open = false)}
          class="inline-flex h-8 w-8 items-center justify-center rounded-md border border-slate-200 text-slate-600 hover:bg-slate-100 dark:border-slate-700 dark:text-slate-300 dark:hover:bg-slate-800"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>
    </div>
    {#if src}
      <iframe {src} {title} class="w-full flex-1 border-0 bg-white dark:bg-slate-900"></iframe>
    {/if}
  </div>
</dialog>
