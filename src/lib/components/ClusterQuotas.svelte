<script lang="ts">
  import { getCluster, QUOTA_GROUPS } from '$lib/content';

  // Rendered inside each cluster page's "Usage Rules" section. Takes the slug
  // rather than the values so a page cannot state a limit that disagrees with
  // its own frontmatter, and so the row ORDER and the labels are identical on
  // all three clusters: the point of the table is that the same six questions
  // are answered in the same places every time.
  let { slug }: { slug: string } = $props();

  const quotas = $derived(getCluster(slug)?.meta.quotas);
</script>

{#if quotas}
  <div
    class="not-prose my-6 overflow-hidden rounded-xl border border-slate-200 bg-slate-50 dark:border-slate-700 dark:bg-slate-800/40"
  >
    {#each QUOTA_GROUPS as group, i}
      <div class={i > 0 ? 'border-t border-slate-200 dark:border-slate-700' : ''}>
        <p
          class="bg-slate-100 px-4 py-2 font-mono text-[0.7rem] font-semibold tracking-[0.14em] text-slate-500 uppercase dark:bg-slate-800/70 dark:text-slate-400"
        >
          {group.title}
        </p>
        <dl class="divide-y divide-slate-200 dark:divide-slate-700">
          {#each group.rows as row}
            <div class="grid grid-cols-1 gap-1 px-4 py-3 sm:grid-cols-[13rem_1fr] sm:gap-6">
              <dt class="pt-0.5 text-sm font-medium text-slate-500 dark:text-slate-400">
                {row.label}
              </dt>
              <dd class="text-slate-700 dark:text-slate-300">{quotas[row.key]}</dd>
            </div>
          {/each}
        </dl>
      </div>
    {/each}
  </div>
{/if}
