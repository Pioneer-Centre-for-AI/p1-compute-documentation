<script lang="ts">
  import Aside from '$lib/components/Aside.svelte';
  import Card from '$lib/components/Card.svelte';
  import CardGrid from '$lib/components/CardGrid.svelte';
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

<section class="not-prose mb-8 flex flex-col items-center gap-4 text-center">
  <picture>
    <source srcset="/assets/logos/PCAI_Solo_White.svg" media="(prefers-color-scheme: dark)" />
    <img src="/assets/logos/PCAI_Solo_RGB.svg" alt="" class="block h-20 w-20 dark:hidden" />
  </picture>
  <img src="/assets/logos/PCAI_Solo_White.svg" alt="" class="hidden h-20 w-20 dark:block" />
  <h1 class="text-4xl font-bold tracking-tight">High-Performance Computing at P1</h1>
  <p class="max-w-2xl text-lg text-slate-600 dark:text-slate-300">
    Your overview guide to high-performance computing resources at the Pioneer Centre for AI in
    Denmark.
  </p>
  <a
    href="/about/"
    class="inline-flex items-center gap-2 rounded-md bg-blue-600 px-5 py-2.5 text-sm font-medium text-white no-underline hover:bg-blue-700"
  >
    Learn More
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><line x1="5" y1="12" x2="19" y2="12"/><polyline points="12,5 19,12 12,19"/></svg>
  </a>
</section>

<Aside type="note" title="Banner">
  The P1 Compute Documentation is not meant to be a comprehensive guide to all things compute. It
  is a starting point for P1 members to get an overview of the compute resources available to them.
  For specific details about the clusters, please refer to the respective cluster support sites.
</Aside>

<p>
  The compute landscape can be complex, and we know it's not always easy for researchers to find
  the right resources. While Denmark offers a wide range of HPC options, we don't aim to cover them
  all here. Instead, as part of the P1 consortium, we focus on collaborating with a select group of
  HPC facilities and operate our own systems in partnership with P1 partner universities — tailored
  specifically to support P1-affiliated research.
</p>

<p>
  We recommend starting by reaching out to your local institution to explore what's available and
  whether it fits your needs. Before diving into the P1 compute resources, please take a moment to
  review our guides — they'll help ensure your projects are set up for compatibility and held to a
  higher standard.
</p>

<ClusterGrid>
  <ClusterCard
    title="General Purpose HPC"
    image="/assets/landing/DTU_324.avif"
    imageAlt="DTU Compute, Building 324, 2800 Kongens Lyngby, Denmark"
    learnMoreLink="/clusters/dtu-hpc/"
  >
    {#snippet children()}
      The <a href="https://www.hpc.dtu.dk/" target="_blank" rel="noopener noreferrer">P1 DTU HPC</a>
      cluster is designed for general research workloads. It provides free compute and storage resources
      for P1 affiliated researchers, making it ideal for a wide range of academic and scientific projects.
      This cluster is not GDPR-compliant and is best suited for non-sensitive data and standard research
      tasks.
    {/snippet}
  </ClusterCard>
  <ClusterCard
    title="Sensitive HPC"
    image="/assets/landing/NGC.avif"
    imageAlt="Danish National Genome Center (NGC)"
    learnMoreLink="/clusters/ngc-hpc/"
    reverse
  >
    {#snippet children()}
      The <a href="https://www.ngc.dk/infrastrukturen-for-personlig-medicin/supercomputer" target="_blank" rel="noopener noreferrer">P1 NGC HPC</a>
      provides a GDPR-compliant computing environment with free compute and storage resources for P1
      affiliated researchers. It is specifically designed for projects that require handling of sensitive
      or personal data, ensuring compliance with data protection regulations while supporting advanced
      research needs.
    {/snippet}
  </ClusterCard>
  <ClusterCard
    title="Large Scale HPC"
    image="/assets/landing/DCAI_Gefion.avif"
    imageAlt="AI supercomputer, Gefion"
    learnMoreLink="/clusters/dcai-gefion/"
  >
    {#snippet children()}
      The <a href="https://www.dcai.dk/gefion" target="_blank" rel="noopener noreferrer">Gefion HPC</a>
      is a partner facility offering pay-per-use compute and storage resources for larger scale projects.
      It is ideal for users with high computational demands that exceed the capacity of the other P1
      clusters.
    {/snippet}
  </ClusterCard>
</ClusterGrid>

<CardGrid>
  <Card title="Intended Use" icon="approve-check">
    {#snippet children()}
      <p>
        Our compute clusters are primarily for research projects. Commercial use is generally not
        permitted on the P1 DTU and NGC HPCs; however, Gefion may accommodate certain commercial
        research projects on a case-by-case basis.
      </p>
    {/snippet}
  </Card>

  <Card title="Resource Policy" icon="license">
    {#snippet children()}
      <p>
        Resources are limited and prioritized for P1 members at the PhD, postdoc, and faculty levels.
        We are committed to open access and strive to serve as many P1 members as possible. For very
        large-scale projects, we encourage you to reach out to discuss potential solutions.
      </p>
    {/snippet}
  </Card>

  <Card title="Detailed Cluster Information" icon="information">
    {#snippet children()}
      <p>
        Each cluster is hosted at a separate institution and managed by its own administrators. This
        site provides an overview and a starting point for accessing P1 compute resources. For the
        most up-to-date technical details, please refer to the respective cluster websites and
        support channels.
      </p>
    {/snippet}
  </Card>

  <Card title="Citation" icon="code-search">
    {#snippet children()}
      <p>
        If you have received support from DCC (the DTU HPC center administering our DTU cluster),
        please cite:
      </p>
      <DigitalObjectIdentifier />
    {/snippet}
  </Card>
</CardGrid>
