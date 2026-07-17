<script lang="ts">
  import { clusters, getCluster } from '$lib/content';
  import { SURVEY_URL } from '$lib/links';

  const decisions = [
    { need: 'General ML/AI research on non-sensitive data', slug: 'dtu', why: 'Free compute and storage, batch + interactive H100 access' },
    { need: 'GDPR-sensitive or personal data', slug: 'ngc', why: 'Air-gapped, GDPR-compliant environment' },
    { need: 'GDPR + springboard to large scale', slug: 'gefion', why: 'DGX H100 sandbox with a path to full 8-GPU nodes' }
  ].map((d) => ({ ...d, cluster: getCluster(d.slug)! }));
</script>

<svelte:head>
  <title>Clusters</title>
  <meta
    name="description"
    content="Three P1 clusters, three research profiles — find the one that fits your project."
  />
</svelte:head>

<div class="mx-auto w-full max-w-3xl">
  <article class="prose">
    <p class="eyebrow not-prose">P1 Compute</p>
    <h1>Clusters</h1>
    <p>
      P1 focuses on three HPC facilities, each matched to a research profile. Use the table below
      to find the cluster that fits your project, then follow its page for access and usage
      instructions.
    </p>
  </article>

  <div class="not-prose my-8 overflow-hidden rounded-xl border border-slate-200 dark:border-slate-700">
    <table class="w-full border-collapse text-sm">
      <thead>
        <tr class="bg-slate-100 text-left dark:bg-slate-800">
          <th class="px-4 py-3 font-semibold">If you need…</th>
          <th class="px-4 py-3 font-semibold">Go to</th>
          <th class="px-4 py-3 font-semibold">Why</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-slate-200 dark:divide-slate-700">
        {#each decisions as d}
          <tr>
            <td class="px-4 py-3 align-top text-slate-700 dark:text-slate-300">{d.need}</td>
            <td class="px-4 py-3 align-top whitespace-nowrap">
              <a href={d.cluster.href} class="font-semibold text-[var(--color-brand)] no-underline hover:underline">
                {d.cluster.meta.navLabel ?? d.cluster.meta.title}
              </a>
            </td>
            <td class="px-4 py-3 align-top text-slate-700 dark:text-slate-300">{d.why}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>

  <div class="mb-8 rounded-xl border border-slate-200 bg-slate-50 p-5 dark:border-slate-700 dark:bg-slate-800/40">
    <p class="eyebrow mb-2">Guidance</p>
    <p class="text-sm leading-relaxed text-slate-600 dark:text-slate-400">
      Unsure which fits, or ready to onboard? Start with the compute coordinator —
      <a
        href={SURVEY_URL}
        target="_blank"
        rel="noopener noreferrer"
        class="font-semibold text-[var(--color-brand)] no-underline hover:underline"
      >tell us about your workload →</a>
      and we'll help you find and use the right resource.
    </p>
  </div>

  <p class="mb-8 text-sm text-slate-600 dark:text-slate-400">
    Open to researchers at any Danish university. Looking beyond P1? See the wider
    <a href="/ecosystem/" class="text-[var(--color-brand)] hover:underline">compute ecosystem</a>.
  </p>

  <div class="grid gap-6 sm:grid-cols-3">
    {#each clusters as cluster}
      <a
        href={cluster.href}
        class="group flex flex-col rounded-xl border border-slate-200 p-5 no-underline transition-colors hover:border-slate-300 hover:bg-slate-50 dark:border-slate-700 dark:hover:border-slate-600 dark:hover:bg-slate-800/50"
      >
        <div class="mb-3 h-[3px] w-10 rounded-sm" style:background-color={cluster.meta.brandColor}></div>
        <div class="mb-1 flex items-center gap-2">
          <span class="font-semibold text-slate-900 dark:text-slate-100">{cluster.meta.title}</span>
          {#if cluster.meta.badge}
            <span class="rounded bg-slate-200 px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-slate-700 dark:bg-slate-700 dark:text-slate-200">
              {cluster.meta.badge.text}
            </span>
          {/if}
        </div>
        <p class="text-sm leading-relaxed text-slate-600 dark:text-slate-400">
          {cluster.meta.description}
        </p>
      </a>
    {/each}
  </div>
</div>
