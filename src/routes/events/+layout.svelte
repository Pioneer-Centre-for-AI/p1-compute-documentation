<script lang="ts">
  import { page } from '$app/stores';
  import BrandAccent from '$lib/components/BrandAccent.svelte';
  import Meta from '$lib/components/Meta.svelte';
  import { headingAnchors } from '$lib/actions/headingAnchors';
  import { getEvents } from '$lib/events';

  let { children } = $props();

  const event = $derived(getEvents().find((e) => e.href === $page.url.pathname) ?? null);

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
          location: {
            '@type': 'Place',
            name: event.meta.venue ?? event.meta.eyebrow,
            ...(event.meta.address ? { address: event.meta.address } : {})
          },
          organizer: {
            '@type': 'Organization',
            name: 'Pioneer Centre for Artificial Intelligence',
            url: 'https://www.aicentre.dk/'
          }
        }).replace(/</g, '\\u003c')
      : ''
  );
</script>

{#if event}
  <Meta title={event.meta.title} description={event.meta.description} type="article" />
{/if}

<svelte:head>
  {#if event}
    {@html `<script type="application/ld+json">${jsonLd}<\/script>`}
  {/if}
</svelte:head>

{#if event}
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
  {@render children()}
{/if}
