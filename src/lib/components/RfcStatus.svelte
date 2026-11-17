<script lang="ts">
  import { STATUS_LABELS, type RfcStatus } from '#lib/rfcs.ts';

  let { status }: { status: RfcStatus } = $props();

  const live = $derived(status === 'open');
</script>

<span
  class="inline-block rounded border px-1.5 py-0.5 font-mono text-[10px] font-medium uppercase tracking-wide {live
    ? 'border-plum text-plum dark:border-plum-bright dark:text-plum-bright'
    : 'border-slate-200 bg-slate-200 text-slate-700 dark:border-slate-700 dark:bg-slate-700 dark:text-slate-200'}"
>
  {STATUS_LABELS[status]}
</span>
