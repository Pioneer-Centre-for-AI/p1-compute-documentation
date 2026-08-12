<script lang="ts">
  import { BOOKING_URL } from '$lib/links';

  // The booking calendar in a modal, so it opens from anywhere on the site
  // without leaving the page. Kept separate from BookConsultation because the
  // header's split button needs the same dialog with a different trigger.
  //
  // A native <dialog> with showModal(), NOT a `fixed inset-0` div, and the
  // difference is load-bearing: the header is `sticky ... backdrop-blur`, and
  // backdrop-filter makes an element the containing block for `fixed`
  // descendants. The split button lives inside that header, so a fixed overlay
  // positioned itself against the header box instead of the viewport and hung
  // off the top of the page. showModal() promotes the dialog to the browser's
  // top layer, which no ancestor transform, filter or z-index can reach.
  let { open = $bindable(false), title = 'Book a consultation' }: { open?: boolean; title?: string } =
    $props();

  let dialog = $state<HTMLDialogElement | null>(null);
  // Empty until first open: the element is always in the DOM (showModal needs
  // it there), and an eager src would load Cal on every page view.
  let src = $state('');

  $effect(() => {
    const el = dialog;
    if (!el) return;
    if (open) {
      if (!el.open) {
        // Cal renders its own chrome, so it has to be told which theme the page
        // is in. Read at open time rather than at mount, so the toggle is
        // respected.
        const theme = document.documentElement.classList.contains('dark') ? 'dark' : 'light';
        src = `${BOOKING_URL}?embed=true&layout=month_view&theme=${theme}`;
        el.showModal();
      }
      // showModal blocks interaction with the page but not always its scroll.
      const prev = document.body.style.overflow;
      document.body.style.overflow = 'hidden';
      return () => {
        document.body.style.overflow = prev;
      };
    }
    if (el.open) el.close();
  });
</script>

<!-- Escape is the dialog's own `cancel`, so `close` is the single place the
     bound state is reset, whichever way it was dismissed. -->
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
  <!-- Cal lays the booker out in three columns (details, month, slots) only
       when the iframe is wide enough; below roughly 1024px it stacks them and
       the dialog reads as a tall scroller. Hence 72rem rather than the 3xl the
       form dialogs use, and a height capped in rem: at a bare 85vh on a tall
       display the three columns float in empty space. -->
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
