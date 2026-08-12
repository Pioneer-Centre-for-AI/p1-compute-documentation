<script lang="ts">
  import { afterNavigate } from '$app/navigation';
  import { fly } from 'svelte/transition';
  import BookingDialog from './BookingDialog.svelte';
  import { SURVEY_PAGE, BOOKING_URL } from '$lib/links';

  // A split button, not a plain dropdown: the survey was the header's one-click
  // action long before booking existed, so the label keeps navigating there and
  // the caret is what reveals the second way to reach the coordinator.
  let open = $state(false);
  let booking = $state(false);
  let wrapper = $state<HTMLElement | null>(null);
  let menu = $state<HTMLElement | null>(null);

  $effect(() => {
    if (!open) return;
    menu?.querySelector<HTMLElement>('a, button')?.focus();
    // Pointerdown rather than click, so the menu is gone before the underlying
    // element reacts, and capture so a stopPropagation elsewhere cannot trap it.
    const onPointerDown = (e: PointerEvent) => {
      if (!wrapper?.contains(e.target as Node)) open = false;
    };
    document.addEventListener('pointerdown', onPointerDown, true);
    return () => document.removeEventListener('pointerdown', onPointerDown, true);
  });

  afterNavigate(() => (open = false));

  function close() {
    open = false;
    wrapper?.querySelector<HTMLElement>('button[aria-haspopup]')?.focus();
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      close();
      return;
    }
    // Arrow keys are what a role="menu" promises; without them the menu is a
    // list of links wearing the wrong role.
    if (e.key !== 'ArrowDown' && e.key !== 'ArrowUp') return;
    if (!open) {
      open = true;
      e.preventDefault();
      return;
    }
    const items = [...(menu?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? [])];
    if (!items.length) return;
    e.preventDefault();
    const at = items.indexOf(document.activeElement as HTMLElement);
    const step = e.key === 'ArrowDown' ? 1 : -1;
    items[(at + step + items.length) % items.length].focus();
  }

  // Both halves of the pill are the same surface, so the hover tint is an
  // overlay rather than a second background colour: two slate shades meeting at
  // the divider read as two buttons that happen to touch.
  const segment =
    'relative cursor-pointer transition-colors before:absolute before:inset-0 before:bg-white/0 before:transition-colors hover:before:bg-white/12 dark:before:bg-slate-900/0 dark:hover:before:bg-slate-900/10 focus-visible:outline-none focus-visible:before:bg-white/12 dark:focus-visible:before:bg-slate-900/10';

  const item =
    'group flex w-full items-start gap-3 rounded-lg px-2.5 py-2 text-left no-underline transition-colors hover:bg-slate-100 focus-visible:bg-slate-100 focus-visible:outline-none dark:hover:bg-slate-800 dark:focus-visible:bg-slate-800';
  const itemIcon =
    'mt-px flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-slate-100 text-slate-500 transition-colors group-hover:bg-white group-hover:text-slate-900 dark:bg-slate-800 dark:text-slate-400 dark:group-hover:bg-slate-700 dark:group-hover:text-slate-100';
</script>

<div bind:this={wrapper} class="relative hidden md:block">
  <div
    class="flex h-9 items-stretch overflow-hidden rounded-full bg-slate-800 text-white ring-1 ring-slate-900/5 ring-inset transition-shadow has-[:focus-visible]:ring-2 has-[:focus-visible]:ring-slate-500 has-[:focus-visible]:ring-offset-2 has-[:focus-visible]:ring-offset-white dark:bg-slate-200 dark:text-slate-900 dark:ring-white/10 dark:has-[:focus-visible]:ring-offset-slate-900"
  >
    <a
      href={SURVEY_PAGE}
      class="{segment} inline-flex items-center pr-3 pl-4 text-sm font-medium whitespace-nowrap text-inherit no-underline"
    >
      <span class="relative">Your Compute Needs</span>
    </a>
    <span class="my-2 w-px shrink-0 bg-current opacity-20"></span>
    <button
      type="button"
      aria-haspopup="menu"
      aria-expanded={open}
      aria-controls="compute-needs-menu"
      aria-label="More ways to reach the compute coordinator"
      onclick={() => (open = !open)}
      onkeydown={onKeydown}
      class="{segment} inline-flex w-9 items-center justify-center {open
        ? 'before:bg-white/12 dark:before:bg-slate-900/10'
        : ''}"
    >
      <svg
        width="13"
        height="13"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2.5"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
        class="relative transition-transform duration-200"
        class:rotate-180={open}
      >
        <polyline points="6 9 12 15 18 9" />
      </svg>
    </button>
  </div>

  {#if open}
    <div
      bind:this={menu}
      id="compute-needs-menu"
      role="menu"
      tabindex="-1"
      onkeydown={onKeydown}
      transition:fly={{ y: -4, duration: 120 }}
      class="absolute right-0 z-40 mt-2 w-80 origin-top-right rounded-2xl border border-slate-200 bg-white p-2 shadow-xl ring-1 ring-slate-900/5 dark:border-slate-700 dark:bg-slate-900 dark:ring-black/40"
    >
      <a href={SURVEY_PAGE} role="menuitem" class={item}>
        <span class={itemIcon}>
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14,2 14,8 20,8" />
            <line x1="8" y1="13" x2="16" y2="13" />
            <line x1="8" y1="17" x2="13" y2="17" />
          </svg>
        </span>
        <span class="min-w-0">
          <span class="block text-sm font-medium text-slate-900 dark:text-slate-100">
            Tell us about your workload
          </span>
          <span class="mt-0.5 block text-xs leading-snug text-slate-500 dark:text-slate-400">
            The compute needs survey
          </span>
        </span>
      </a>
      <button
        type="button"
        role="menuitem"
        onclick={() => {
          open = false;
          booking = true;
        }}
        class={item}
      >
        <span class={itemIcon}>
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <rect x="3" y="4" width="18" height="18" rx="2" />
            <line x1="16" y1="2" x2="16" y2="6" />
            <line x1="8" y1="2" x2="8" y2="6" />
            <line x1="3" y1="10" x2="21" y2="10" />
          </svg>
        </span>
        <span class="min-w-0">
          <span class="block text-sm font-medium text-slate-900 dark:text-slate-100">
            Book a consultation
          </span>
          <span class="mt-0.5 block text-xs leading-snug text-slate-500 dark:text-slate-400">
            Meet the compute coordinator, in person or online
          </span>
        </span>
      </button>
      <div class="mx-2.5 my-1.5 border-t border-slate-200 dark:border-slate-700"></div>
      <a
        href={BOOKING_URL}
        target="_blank"
        rel="noopener noreferrer"
        role="menuitem"
        class="flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-xs text-slate-500 no-underline transition-colors hover:bg-slate-100 hover:text-slate-900 focus-visible:bg-slate-100 focus-visible:outline-none dark:text-slate-400 dark:hover:bg-slate-800 dark:hover:text-slate-100"
      >
        Open the calendar in a new tab
        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M15 3h6v6M10 14L21 3M21 14v7H3V3h7" />
        </svg>
      </a>
    </div>
  {/if}
</div>

<BookingDialog bind:open={booking} />
