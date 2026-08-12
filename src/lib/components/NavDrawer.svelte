<script lang="ts">
  import { afterNavigate } from '$app/navigation';
  import NavList from './NavList.svelte';
  import BookingDialog from './BookingDialog.svelte';
  import { SURVEY_PAGE, PROFILE_URL } from '$lib/links';

  let { open = $bindable(false) }: { open?: boolean } = $props();
  let panel = $state<HTMLElement | null>(null);
  let booking = $state(false);

  $effect(() => {
    if (!open) return;
    const prev = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    panel?.querySelector<HTMLElement>('a, button')?.focus();
    return () => {
      document.body.style.overflow = prev;
    };
  });

  afterNavigate(() => (open = false));

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      open = false;
      return;
    }
    if (e.key !== 'Tab' || !panel) return;
    const els = panel.querySelectorAll<HTMLElement>('a[href], button:not([disabled])');
    if (!els.length) return;
    const first = els[0];
    const last = els[els.length - 1];
    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault();
      first.focus();
    }
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="fixed inset-0 z-40 md:hidden"
    role="dialog"
    aria-modal="true"
    aria-label="Site navigation"
    tabindex="-1"
    onkeydown={onKeydown}
  >
    <button
      aria-label="Close menu"
      onclick={() => (open = false)}
      class="absolute inset-0 h-full w-full cursor-default bg-slate-950/50 backdrop-blur-sm"
    ></button>
    <div
      bind:this={panel}
      class="absolute inset-y-0 left-0 w-72 max-w-[85vw] overflow-y-auto bg-white p-4 shadow-xl dark:bg-slate-900"
    >
      <div class="mb-4 flex items-center justify-between">
        <span class="eyebrow">Menu</span>
        <button
          aria-label="Close menu"
          onclick={() => (open = false)}
          class="inline-flex h-9 w-9 items-center justify-center rounded-md border border-slate-200 text-slate-700 hover:bg-slate-100 dark:border-slate-700 dark:text-slate-200 dark:hover:bg-slate-800"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>
      <NavList />
      <div class="mt-6 flex flex-col gap-2 border-t border-slate-200 pt-4 dark:border-slate-700">
        <a
          href={SURVEY_PAGE}
          class="inline-flex h-9 items-center justify-center rounded-full bg-slate-800 px-4 text-sm font-medium text-white no-underline transition-colors hover:bg-slate-700 dark:bg-slate-200 dark:text-slate-900 dark:hover:bg-slate-300"
        >
          Your Compute Needs
        </a>
        <button
          type="button"
          onclick={() => {
            open = false;
            booking = true;
          }}
          class="inline-flex h-9 items-center justify-center rounded-full border border-slate-400 px-4 text-sm font-medium text-slate-700 transition-colors hover:bg-slate-100 dark:border-slate-600 dark:text-slate-200 dark:hover:bg-slate-800"
        >
          Book a consultation
        </button>
        <a
          href={PROFILE_URL}
          target="_blank"
          rel="noopener noreferrer"
          class="inline-flex h-9 items-center justify-center rounded-full border border-slate-400 px-4 text-sm font-medium text-slate-700 no-underline transition-colors hover:bg-slate-100 dark:border-slate-600 dark:text-slate-200 dark:hover:bg-slate-800"
        >
          Update Profile
        </a>
      </div>
    </div>
  </div>
{/if}

<!-- Outside the drawer's own `{#if}`: the button that opens it also closes the
     drawer, so a dialog nested inside would be destroyed on the same click. -->
<BookingDialog bind:open={booking} />
