<script lang="ts">
  import Button from '$lib/components/Button.svelte';
  import DigitalObjectIdentifier from '$lib/components/DigitalObjectIdentifier.svelte';
  import { clusters } from '$lib/content';
  import { SURVEY_URL } from '$lib/links';

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
      profile: 'Large Scale',
      image: '/assets/landing/DCAI_Gefion.avif',
      imageAlt: 'AI supercomputer, Gefion'
    }
  };

  const ecosystem = ['DeiC', 'LUMI', 'UCloud', 'GenomeDK', 'Computerome'];
</script>

<svelte:head>
  <title>P1 Compute Documentation</title>
  <meta
    name="description"
    content="Your overview guide to high-performance computing resources at the Pioneer Centre for AI in Denmark."
  />
</svelte:head>

<main class="mx-auto w-full max-w-6xl px-4 md:px-6">
  <!-- Hero -->
  <section class="grid items-center gap-12 pb-20 pt-32 md:pt-40 lg:grid-cols-[1.2fr_1fr] lg:gap-8">
    <div class="hero-reveal flex flex-col items-start gap-6">
      <p class="eyebrow">Pioneer Centre for AI · Compute</p>
      <h1 class="text-balance text-5xl font-extrabold leading-[1.05] tracking-tight text-slate-900 md:text-7xl dark:text-slate-50">
        Compute that fits your research.
      </h1>
      <p class="max-w-xl text-lg leading-relaxed text-slate-500 dark:text-slate-400">
        Three HPC clusters, three research profiles — free for P1-affiliated researchers at Danish
        universities. We run them, and we point the way to the rest of the landscape.
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
    </div>

    <!-- Network motif: you, connected to three clusters -->
    <svg
      viewBox="0 0 400 340"
      aria-hidden="true"
      class="hero-reveal mx-auto w-full max-w-sm lg:max-w-none"
      style="animation-delay: 0.15s"
    >
      <g stroke="currentColor" stroke-width="1.25" class="text-slate-300 dark:text-slate-600">
        <line x1="200" y1="185" x2="200" y2="52" />
        <line x1="200" y1="185" x2="72" y2="278" />
        <line x1="200" y1="185" x2="328" y2="278" />
      </g>

      <!-- you -->
      <circle cx="200" cy="185" r="9" fill="var(--color-coral)" />
      <circle cx="200" cy="185" r="9" fill="none" stroke="var(--color-coral)" class="pulse-ring" />
      <text x="200" y="215" text-anchor="middle" class="node-label">you</text>

      <!-- clusters -->
      <circle cx="200" cy="44" r="12" fill="var(--color-brand-dtu)" />
      <text x="200" y="24" text-anchor="middle" class="node-label">dtu</text>

      <circle cx="64" cy="284" r="12" fill="var(--color-brand-ngc)" />
      <text x="64" y="316" text-anchor="middle" class="node-label">ngc</text>

      <circle cx="336" cy="284" r="12" fill="var(--color-brand-gefion)" />
      <text x="336" y="316" text-anchor="middle" class="node-label">gefion</text>
    </svg>
  </section>

  <!-- Three clusters, three profiles -->
  <section class="border-t border-slate-200 py-20 dark:border-slate-800">
    <div class="mb-10 flex items-baseline gap-4">
      <span class="eyebrow">01 · Clusters</span>
      <h2 class="text-2xl font-semibold text-slate-900 md:text-3xl dark:text-slate-50">
        Three clusters, three research profiles.
      </h2>
    </div>

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

    <p class="mt-10 text-sm text-slate-500 dark:text-slate-400">
      Not sure which one fits your project?
      <a href="/clusters/" class="font-semibold text-[var(--color-brand)] no-underline hover:underline">
        Compare the clusters →
      </a>
    </p>
  </section>

  <!-- Beyond P1 -->
  <section class="border-t border-slate-200 py-20 dark:border-slate-800">
    <div class="grid gap-8 lg:grid-cols-[1fr_auto] lg:items-center">
      <div class="flex max-w-2xl flex-col gap-4">
        <span class="eyebrow">02 · Ecosystem</span>
        <h2 class="text-2xl font-semibold text-slate-900 md:text-3xl dark:text-slate-50">
          Beyond P1 — we point the way.
        </h2>
        <p class="leading-relaxed text-slate-500 dark:text-slate-400">
          The Danish compute landscape is bigger than our three clusters. When your needs outgrow
          P1, we help you find the right national or international resource. We consult and guide —
          we don't broker access.
        </p>
        <a href="/ecosystem/" class="text-sm font-semibold text-[var(--color-brand)] no-underline hover:underline">
          Explore the ecosystem →
        </a>
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

  <!-- Guidance -->
  <section class="border-t border-slate-200 py-20 dark:border-slate-800">
    <div class="flex max-w-2xl flex-col items-start gap-4">
      <span class="eyebrow">03 · Guidance</span>
      <h2 class="text-2xl font-semibold text-slate-900 md:text-3xl dark:text-slate-50">
        Start with a conversation.
      </h2>
      <p class="leading-relaxed text-slate-500 dark:text-slate-400">
        Onboarding goes through the P1 compute coordinator. Tell us about your workload, needs,
        and expectations — it helps us point you to the right resource and lift your ability to
        use it well.
      </p>
      <Button
        href={SURVEY_URL}
        variant="primary"
        target="_blank"
        rel="noopener noreferrer"
      >
        {#snippet children()}
          Tell us about your workload
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M15 3h6v6M10 14L21 3M21 14v7H3V3h7" /></svg>
        {/snippet}
      </Button>
    </div>
  </section>

  <!-- Citation -->
  <section class="border-t border-slate-200 py-16 dark:border-slate-800">
    <div class="flex flex-col items-start gap-4">
      <span class="eyebrow">04 · Cite us</span>
      <p class="max-w-2xl text-sm leading-relaxed text-slate-500 dark:text-slate-400">
        If you've received support from the DTU HPC centre administering our DTU cluster, please
        cite:
      </p>
      <DigitalObjectIdentifier />
    </div>
  </section>
</main>

<style>
  .node-label {
    font-family: var(--font-mono);
    font-size: 11px;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    fill: var(--color-slate-500);
  }

  .pulse-ring {
    transform-origin: 200px 185px;
    animation: pulse 2.4s cubic-bezier(0.4, 0, 0.2, 1) infinite;
  }

  .hero-reveal {
    animation: fade-up 0.6s ease-out both;
  }

  @keyframes pulse {
    0% {
      transform: scale(1);
      opacity: 0.8;
    }
    100% {
      transform: scale(2.6);
      opacity: 0;
    }
  }

  @keyframes fade-up {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .pulse-ring,
    .hero-reveal {
      animation: none;
    }
  }
</style>
