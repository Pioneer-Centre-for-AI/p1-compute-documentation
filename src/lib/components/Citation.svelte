<script lang="ts">
  import DigitalObjectIdentifier from './DigitalObjectIdentifier.svelte';
</script>

<aside
  class="not-prose my-6 rounded-xl border border-slate-200 bg-slate-50 p-5 dark:border-slate-700 dark:bg-slate-800/40"
>
  <p class="eyebrow mb-2">Citing this cluster</p>
  <p class="mb-4 text-sm leading-relaxed text-slate-600 dark:text-slate-400">
    If the P1 DTU HPC contributed to your work, please cite the DTU Computing Center resources DOI.
    That covers help from the DTU DCC team or the P1 compute coordinator just as much as time spent
    running jobs, and it is the clearest evidence we have that these resources are worth sustaining.
  </p>
  <DigitalObjectIdentifier />
</aside>
