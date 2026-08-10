<script lang="ts">
  import { page } from '$app/stores';
  import BrandAccent from '$lib/components/BrandAccent.svelte';
  import { headingAnchors } from '$lib/actions/headingAnchors';
  import { events } from '$lib/events';

  let { children } = $props();

  // Null on /events/ itself, which is the overview page rather than an event.
  // It renders its own <article> and passes through the {:else} branch below.
  const event = $derived(events.find((e) => e.href === $page.url.pathname) ?? null);

  // schema.org Event, so calendars and search surfaces can read the session.
  // The room is not fixed yet, so location carries the conference name only.
  const jsonLd = $derived(
    event
      ? JSON.stringify({
          '@context': 'https://schema.org',
          '@type': 'Event',
          name: event.meta.title,
          description: event.meta.description,
          startDate: event.meta.startDate,
          ...(event.meta.duration ? { duration: event.meta.duration } : {}),
          eventAttendanceMode: 'https://schema.org/OfflineEventAttendanceMode',
          location: { '@type': 'Place', name: event.meta.eyebrow },
          organizer: {
            '@type': 'Organization',
            name: 'Pioneer Centre for Artificial Intelligence',
            url: 'https://www.aicentre.dk/'
          }
        }).replace(/</g, '\\u003c')
      : ''
  );
</script>

<svelte:head>
  {#if event}
    <title>{event.meta.title}</title>
    <meta name="description" content={event.meta.description} />
    <!-- Built in markup, not in the script block above: a literal closing
         script tag inside <script> would terminate it. -->
    {@html `<script type="application/ld+json">${jsonLd}<\/script>`}
  {/if}
</svelte:head>

{#if event}
  <!-- The TOC aside and the two-column shell come from the root layout. -->
  <article class="prose mx-auto w-full max-w-3xl" use:headingAnchors>
    <div class="not-prose">
      <div class="mb-4 flex items-center gap-3">
        <BrandAccent color="var(--color-plum)" width="w-12" />
        <span class="eyebrow">Event · {event.meta.eyebrow}</span>
      </div>
      <h1
        class="mb-0 mt-0 text-4xl font-extrabold tracking-tight text-slate-900 dark:text-slate-50"
      >
        {event.meta.title}
      </h1>
    </div>
    {@render children()}
  </article>
{:else}
  <!-- /events/ overview: brings its own <article> and <svelte:head>. -->
  {@render children()}
{/if}
