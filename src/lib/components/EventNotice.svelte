<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { upcomingEvent } from '#lib/events.ts';

  const event = upcomingEvent();
  const wideLabel = event
    ? [event.meta.dateLabel, event.meta.venue, event.meta.title].filter(Boolean).join(' · ')
    : '';
  let stale = $state(false);

  onMount(() => {
    if (event && new Date().toISOString().slice(0, 10) > event.meta.startDate) stale = true;
  });
</script>

{#if event && !stale && page.url.pathname !== event.href}
  <a
    href={event.href}
    class="group flex items-center justify-center gap-x-2 gap-y-0.5 border-b border-slate-200 bg-slate-100 px-4 py-1.5 text-center text-xs no-underline dark:border-slate-700 dark:bg-slate-800"
  >
    <span class="eyebrow shrink-0">Upcoming</span>
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
