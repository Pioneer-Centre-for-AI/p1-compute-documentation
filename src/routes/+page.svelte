<script lang="ts">
  import Button from '$lib/components/Button.svelte';
  import ClusterCard from '$lib/components/ClusterCard.svelte';
  import ClusterGrid from '$lib/components/ClusterGrid.svelte';
  import DigitalObjectIdentifier from '$lib/components/DigitalObjectIdentifier.svelte';
</script>

<svelte:head>
  <title>P1 Compute Documentation</title>
  <meta
    name="description"
    content="Your overview guide to high-performance computing resources at the Pioneer Centre for AI in Denmark."
  />
</svelte:head>

<main class="mx-auto flex w-full max-w-5xl flex-col gap-12 px-4 py-16 md:px-6 md:py-24">
  <section class="flex flex-col items-center gap-4 text-center">
    <img src="/assets/logos/PCAI_Solo_RGB.svg" alt="" class="block h-20 w-20 dark:hidden" />
    <img src="/assets/logos/PCAI_Solo_White.svg" alt="" class="hidden h-20 w-20 dark:block" />
    <h1 class="text-4xl font-bold tracking-tight md:text-5xl">High-Performance Computing at P1</h1>
    <p class="max-w-2xl text-lg text-slate-600 dark:text-slate-300">
      Your overview guide to high-performance computing resources at the Pioneer Centre for AI in
      Denmark.
    </p>
    <Button href="/about/" variant="primary">
      {#snippet children()}
        Learn More
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><line x1="5" y1="12" x2="19" y2="12"/><polyline points="12,5 19,12 12,19"/></svg>
      {/snippet}
    </Button>
  </section>

  <section class="flex flex-col items-center gap-3 text-center">
    <p class="text-sm text-slate-600 dark:text-slate-400">
      If you've received support from the DTU HPC centre administering our DTU cluster, please cite:
    </p>
    <DigitalObjectIdentifier />
  </section>

  <section class="flex flex-col gap-8">
    <p class="mx-auto max-w-3xl text-center text-slate-700 dark:text-slate-300">
      The compute landscape can be complex, and we know it's not always easy for researchers to
      find the right resources. As part of the P1 consortium, we focus on a select group of HPC
      facilities tailored to support P1-affiliated research.
    </p>

    <p class="mx-auto -mt-4 text-center text-sm text-slate-600 dark:text-slate-400">
      Not sure which cluster fits your project?
      <a href="/clusters/" class="font-semibold text-[var(--color-brand)] hover:underline">Compare clusters →</a>
    </p>

    <ClusterGrid>
      <ClusterCard
        title="General Purpose HPC"
        image="/assets/landing/DTU_324.avif"
        imageAlt="DTU Compute, Building 324, 2800 Kongens Lyngby, Denmark"
        learnMoreLink="/clusters/dtu/"
      >
        {#snippet children()}
          The <a href="https://www.hpc.dtu.dk/" target="_blank" rel="noopener noreferrer">P1 DTU HPC</a>
          cluster is designed for general research workloads. It provides free compute and storage
          resources for P1 affiliated researchers — ideal for a wide range of academic and
          scientific projects. Not GDPR-compliant; best suited for non-sensitive data and standard
          research tasks.
        {/snippet}
      </ClusterCard>
      <ClusterCard
        title="Sensitive HPC"
        image="/assets/landing/NGC.avif"
        imageAlt="Danish National Genome Center (NGC)"
        learnMoreLink="/clusters/ngc/"
        reverse
      >
        {#snippet children()}
          The <a href="https://www.ngc.dk/infrastrukturen-for-personlig-medicin/supercomputer" target="_blank" rel="noopener noreferrer">P1 NGC HPC</a>
          provides a GDPR-compliant computing environment with free compute and storage resources
          for P1 affiliated researchers. Designed for projects that require handling of sensitive
          or personal data.
        {/snippet}
      </ClusterCard>
      <ClusterCard
        title="Large Scale HPC"
        image="/assets/landing/DCAI_Gefion.avif"
        imageAlt="AI supercomputer, Gefion"
        learnMoreLink="/clusters/gefion/"
      >
        {#snippet children()}
          The <a href="https://www.dcai.dk/gefion" target="_blank" rel="noopener noreferrer">Gefion HPC</a>
          is a partner facility offering pay-per-use compute and storage resources for larger-scale
          projects — ideal for users with high computational demands that exceed the capacity of
          the other P1 clusters.
        {/snippet}
      </ClusterCard>
    </ClusterGrid>
  </section>
</main>
