<script lang="ts">
  type Props = {
    title: string;
    src: string;
    /** Clean URL for opening the form in its own tab; defaults to `src` with embed params stripped. */
    href?: string;
    description?: string;
  };
  let { title, src, href, description }: Props = $props();

  const newTabUrl = $derived(
    href ??
      src
        .replace(/[?&](embedded|embed)=true/gi, (m) => (m.startsWith('?') ? '?' : ''))
        .replace(/\?$/, '')
  );

  // A native <dialog> with showModal(), NOT a `fixed inset-0` div. Any ancestor
  // with a transform, filter or backdrop-filter becomes the containing block
  // for `fixed` descendants, which is what put the booking modal half off the
  // top of the page when it opened from the blurred sticky header. Nothing
  // renders this component from inside such an ancestor today; using the top
  // layer means nothing has to keep checking.
  let open = $state(false);
  let dialog = $state<HTMLDialogElement | null>(null);
  // Empty until first open: the element is always in the DOM (showModal needs
  // it there), and an eager src would load the form on every page view.
  let frameSrc = $state('');

  $effect(() => {
    const el = dialog;
    if (!el) return;
    if (open) {
      if (!el.open) {
        frameSrc = src;
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
      Fill in
    </button>
    <a
      href={newTabUrl}
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

<!-- Escape is the dialog's own `cancel`, so `close` is the single place the
     state is reset, whichever way it was dismissed. -->
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
    class="flex h-[min(85vh,46rem)] w-[min(48rem,100vw-2rem)] flex-col overflow-hidden rounded-xl bg-white shadow-2xl dark:bg-slate-900"
  >
    <div class="flex items-center justify-between gap-3 border-b border-slate-200 px-4 py-3 dark:border-slate-700">
      <p class="min-w-0 truncate text-sm font-semibold text-slate-900 dark:text-slate-100">{title}</p>
      <div class="flex shrink-0 items-center gap-2">
        <a
          href={newTabUrl}
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
          aria-label="Close form"
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
    {#if frameSrc}
      <iframe src={frameSrc} {title} class="w-full flex-1 border-0 bg-white"></iframe>
    {/if}
  </div>
</dialog>
