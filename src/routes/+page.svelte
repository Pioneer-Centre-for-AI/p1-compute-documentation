<script lang="ts">
  import Button from '$lib/components/Button.svelte';
  import Meta from '$lib/components/Meta.svelte';
  import DigitalObjectIdentifier from '$lib/components/DigitalObjectIdentifier.svelte';
  import LandscapeDiagram from '$lib/components/LandscapeDiagram.svelte';
  import { clusters } from '$lib/content';
  import { SURVEY_PAGE } from '$lib/links';

  const profiles: Record<string, { profile: string; image: string; imageAlt: string }> = {
    dtu: {
      profile: 'General Purpose',
      image: '/assets/landing/DTU_324.avif',
      imageAlt: 'DTU Compute, Building 324, 2800 Kongens Lyngby, Denmark'
    },
    ngc: {
      profile: 'Sensitive Data',
      image: '/assets/landing/NGC.avif',
      imageAlt: 'Danish National Genome Center (NGC)'
    },
    gefion: {
      profile: 'Path to Scale',
      image: '/assets/landing/DCAI_Gefion.avif',
      imageAlt: 'AI supercomputer, Gefion'
    }
  };

  const ecosystem = ['DeiC', 'LUMI', 'UCloud', 'GenomeDK', 'Computerome'];
</script>

<Meta
  title="Pioneer Centre for AI · Compute"
  description="Overview, guidance, and onboarding for high-performance computing at the Pioneer Centre for AI in Denmark."
  isLanding
/>

<main class="mx-auto w-full max-w-6xl px-4 md:px-6">
  <!-- Hero -->
  <section class="grid items-center gap-12 pb-14 pt-28 md:pt-36 lg:grid-cols-[1.05fr_1fr] lg:gap-10">
    <div class="flex flex-col items-start gap-6">
      <p class="eyebrow">Overview · Guidance · Onboarding</p>
      <h1 class="text-balance text-5xl font-extrabold leading-[1.05] tracking-tight text-slate-900 md:text-6xl dark:text-slate-50">
        Your <mark>entry point</mark> to research compute.
      </h1>
      <p class="max-w-xl text-lg leading-relaxed text-slate-500 dark:text-slate-400">
        Getting compute for research can be challenging. At P1 we are well aware. We fund and help
        run dedicated partitions at three Danish HPC facilities, free to use for P1-affiliated
        researchers, with our compute coordinator working alongside each provider's HPC team to keep
        the service fit for research. When your needs fit better elsewhere, we know the landscape and
        point you in the right direction.
      </p>
      <div class="flex flex-wrap items-center gap-3">
        <Button href="/clusters/" variant="primary">
          {#snippet children()}
            Find your cluster
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><line x1="5" y1="12" x2="19" y2="12"/><polyline points="12,5 19,12 12,19"/></svg>
          {/snippet}
        </Button>
        <Button href="/about/" variant="ghost">
          {#snippet children()}
            About P1 compute
          {/snippet}
        </Button>
      </div>
      <div class="flex flex-wrap items-center gap-3 pt-2">
        <span class="text-xs text-slate-500 dark:text-slate-400">
          Did the P1 DTU HPC help your work? Please cite:
        </span>
        <DigitalObjectIdentifier />
      </div>
    </div>

    <!-- The compute landscape: P1 as entry point -->
    <div class="mx-auto w-full max-w-md lg:max-w-none">
      <LandscapeDiagram variant="stacked" />
      <p class="mt-3 text-center text-sm text-slate-500 dark:text-slate-400">
        We fund and help run partitions at partner facilities, and we point the way to the rest. We
        consult and guide; we don't broker access.
      </p>
    </div>
  </section>

  <!-- Three clusters, three profiles -->
  <section class="border-t border-slate-200 py-14 dark:border-slate-800">
    <h2 class="mb-8 text-2xl font-semibold text-slate-900 md:text-3xl dark:text-slate-50">
      Three partitions, three research profiles.
    </h2>

    <div class="grid gap-8 md:grid-cols-3">
      {#each clusters as cluster}
        {@const p = profiles[cluster.slug]}
        <a
          href={cluster.href}
          class="group flex flex-col overflow-hidden rounded-xl border border-slate-200 no-underline transition-all hover:-translate-y-0.5 hover:shadow-lg dark:border-slate-700"
        >
          {#if p}
            <img
              src={p.image}
              alt={p.imageAlt}
              loading="lazy"
              class="aspect-video w-full object-cover"
            />
          {/if}
          <div class="h-[3px] w-full" style:background-color={cluster.meta.brandColor}></div>
          <div class="flex flex-1 flex-col gap-2 p-5">
            {#if p}
              <span class="eyebrow">{p.profile}</span>
            {/if}
            <span class="flex items-center gap-2 text-lg font-semibold text-slate-900 dark:text-slate-100">
              {cluster.meta.title}
              {#if cluster.meta.badge}
                <span class="rounded bg-slate-100 px-1.5 py-0.5 font-mono text-[10px] font-medium uppercase tracking-wide text-slate-600 dark:bg-slate-800 dark:text-slate-300">
                  {cluster.meta.badge.text}
                </span>
              {/if}
            </span>
            <p class="flex-1 text-sm leading-relaxed text-slate-500 dark:text-slate-400">
              {cluster.meta.description}
            </p>
            <span class="mt-2 text-sm font-semibold text-[var(--color-brand)] group-hover:underline">
              Learn more →
            </span>
          </div>
        </a>
      {/each}
    </div>

    <p class="mt-8 text-sm text-slate-500 dark:text-slate-400">
      Not sure which one fits your project?
      <a href="/clusters/" class="font-semibold text-[var(--color-brand)] no-underline hover:underline">
        Compare the clusters →
      </a>
    </p>
  </section>

  <!-- Guidance -->
  <section class="border-t border-slate-200 py-14 dark:border-slate-800">
    <div class="grid gap-8 lg:grid-cols-[1fr_auto] lg:items-center">
      <div class="flex max-w-2xl flex-col items-start gap-4">
        <h2 class="text-2xl font-semibold text-slate-900 md:text-3xl dark:text-slate-50">
          Happy to talk it through.
        </h2>
        <p class="leading-relaxed text-slate-500 dark:text-slate-400">
          The P1 compute coordinator is here whenever you want a hand. Tell us about your workload,
          needs, and expectations and we can point you to the right resource and lift your ability
          to use it well. If your project outgrows P1, the wider
          <a href="/ecosystem/" class="font-semibold text-[var(--color-brand)] no-underline hover:underline">compute ecosystem</a>
          is the next step, and we help you choose.
        </p>
        <Button
          href={SURVEY_PAGE}
          variant="primary"
        >
          {#snippet children()}
            Tell us about your workload
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M15 3h6v6M10 14L21 3M21 14v7H3V3h7" /></svg>
          {/snippet}
        </Button>
      </div>
      <ul class="flex flex-wrap gap-2 lg:max-w-xs lg:justify-end">
        {#each ecosystem as name}
          <li>
            <span class="inline-block rounded-full border border-slate-200 px-3 py-1 font-mono text-xs text-slate-600 dark:border-slate-700 dark:text-slate-300">
              {name}
            </span>
          </li>
        {/each}
      </ul>
    </div>
  </section>
</main>
