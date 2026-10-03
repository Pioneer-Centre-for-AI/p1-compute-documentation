<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { upcomingEvent } from '$lib/events';

  // Evaluated once at build time so the prerendered HTML and the first client
  // render agree. A build that goes stale past the event date is caught after
  // hydration below rather than by a mismatched {#if}.
  const event = upcomingEvent();
  // When, where, what: the venue sits before the title, so on a narrow
  // desktop it is the title that truncates, not the room.
  const wideLabel = event
    ? [event.meta.dateLabel, event.meta.venue, event.meta.title].filter(Boolean).join(' · ')
    : '';
  let stale = $state(false);

  onMount(() => {
    if (event && new Date().toISOString().slice(0, 10) > event.meta.startDate) stale = true;
  });
</script>

{#if event && !stale && $page.url.pathname !== event.href}
  <a
    href={event.href}
    class="group flex items-center justify-center gap-x-2 gap-y-0.5 border-b border-slate-200 bg-slate-100 px-4 py-1.5 text-center text-xs no-underline dark:border-slate-700 dark:bg-slate-800"
  >
    <span class="eyebrow shrink-0">Upcoming</span>
    <!-- The full title truncates to nothing on a phone, so narrow screens get
         the compact label instead of a cut-off sentence. The whole label
         underlines on hover, so the bar reads as one link. -->
    <span
      class="min-w-0 truncate text-slate-600 decoration-[var(--color-brand)] underline-offset-2 group-hover:underline dark:text-slate-300"
    >
      <span class="sm:hidden">
        {event.meta.shortLabel ?? event.meta.navLabel ?? event.meta.title}
      </span>
      <span class="hidden sm:inline">{wideLabel}</span>
    </span>
    <span
      class="shrink-0 font-semibold text-[var(--color-brand)] transition-transform group-hover:translate-x-0.5"
      aria-hidden="true">&rarr;</span
    >
  </a>
{/if}
