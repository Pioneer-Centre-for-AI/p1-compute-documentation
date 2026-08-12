<script lang="ts">
  import { BOOKING_URL } from '$lib/links';

  // The booking calendar in a modal, so it opens from anywhere on the site
  // without leaving the page. Kept separate from BookConsultation because the
  // header's split button needs the same dialog with a different trigger.
  let { open = $bindable(false), title = 'Book a consultation' }: { open?: boolean; title?: string } =
    $props();

  let panel = $state<HTMLElement | null>(null);
  // Cal renders its own chrome, so it has to be told which theme the page is
  // in. Read at open time rather than at mount, so the toggle is respected.
  let src = $state(BOOKING_URL);

  $effect(() => {
    if (!open) return;
    const theme = document.documentElement.classList.contains('dark') ? 'dark' : 'light';
    src = `${BOOKING_URL}?embed=true&layout=month_view&theme=${theme}`;
    const prev = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    panel?.querySelector<HTMLElement>('a, button')?.focus();
    return () => {
      document.body.style.overflow = prev;
    };
  });

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') open = false;
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4"
    role="dialog"
    aria-modal="true"
    aria-label={title}
    tabindex="-1"
    onkeydown={onKeydown}
  >
    <button
      aria-label="Close booking"
      onclick={() => (open = false)}
      class="absolute inset-0 h-full w-full cursor-default bg-slate-950/50 backdrop-blur-sm"
    ></button>
    <div
      bind:this={panel}
      class="relative flex h-[85vh] w-full max-w-3xl flex-col overflow-hidden rounded-xl bg-white shadow-2xl dark:bg-slate-900"
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
      <iframe {src} {title} class="w-full flex-1 border-0 bg-white dark:bg-slate-900"></iframe>
    </div>
  </div>
{/if}
