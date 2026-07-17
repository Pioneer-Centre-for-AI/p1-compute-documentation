<script lang="ts">
  type Spec = { label: string; value: string };
  type Props = { items: Spec[] };
  let { items }: Props = $props();
</script>

<dl class="not-prose my-6 divide-y divide-slate-200 overflow-hidden rounded-xl border border-slate-200 bg-slate-50 dark:divide-slate-700 dark:border-slate-700 dark:bg-slate-800/40">
  {#each items as item}
    <div class="grid grid-cols-1 gap-1 px-4 py-3 sm:grid-cols-[13rem_1fr] sm:gap-6">
      <dt class="pt-0.5 font-mono text-xs font-medium uppercase tracking-[0.14em] text-slate-500 dark:text-slate-400">{item.label}</dt>
      <dd class="text-slate-700 dark:text-slate-300">{item.value}</dd>
    </div>
  {/each}
</dl>
