<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { upcomingEvent } from '$lib/events';

  // Evaluated once at build time so the prerendered HTML and the first client
  // render agree. A build that goes stale past the event date is caught after
  // hydration below rather than by a mismatched {#if}.
  const event = upcomingEvent();
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
         the compact label instead of a cut-off sentence. -->
    <span class="min-w-0 truncate text-slate-600 dark:text-slate-300">
      <span class="sm:hidden">
        {event.meta.shortLabel ?? event.meta.navLabel ?? event.meta.title}
      </span>
      <span class="hidden sm:inline">{event.meta.dateLabel} · {event.meta.title}</span>
    </span>
    <span
      class="shrink-0 font-semibold text-[var(--color-brand)] group-hover:underline"
      aria-hidden="true">&rarr;</span
    >
  </a>
{/if}
