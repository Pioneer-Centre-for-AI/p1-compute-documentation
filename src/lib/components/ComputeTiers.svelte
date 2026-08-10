<script lang="ts">
  import { computeTiers } from '$lib/content';
</script>

<!-- Cards below sm, table above it. As a plain markdown table the description
     column collapsed to roughly a dozen characters wide on a phone. The
     descriptions carry authored inline HTML for the example-facility links; see
     the note on ComputeTier in $lib/content. -->

<div class="not-prose my-6 flex flex-col gap-3 sm:hidden">
  {#each computeTiers as t}
    <div
      class="rounded-xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-700 dark:bg-slate-800/40"
    >
      <div class="mb-2 flex items-baseline gap-2">
        <span class="font-mono text-2xl font-semibold text-slate-400 tabular-nums dark:text-slate-500"
          >{t.tier}</span
        >
        <span class="font-semibold text-slate-900 dark:text-slate-100">{t.level}</span>
      </div>
      <p class="text-sm leading-relaxed text-slate-600 dark:text-slate-400">{@html t.description}</p>
    </div>
  {/each}
</div>

<div class="not-prose my-6 hidden overflow-x-auto sm:block">
  <table class="w-full border-collapse text-sm">
    <thead>
      <tr class="bg-slate-100 text-left dark:bg-slate-800">
        <th class="border border-slate-200 px-4 py-3 font-semibold dark:border-slate-700">Tier</th>
        <th class="border border-slate-200 px-4 py-3 font-semibold dark:border-slate-700"
          >Organization level</th
        >
        <th class="border border-slate-200 px-4 py-3 font-semibold dark:border-slate-700"
          >Description</th
        >
      </tr>
    </thead>
    <tbody>
      {#each computeTiers as t}
        <tr>
          <td
            class="border border-slate-200 px-4 py-3 align-top font-mono tabular-nums text-slate-700 dark:border-slate-700 dark:text-slate-300"
            >{t.tier}</td
          >
          <td
            class="border border-slate-200 px-4 py-3 align-top whitespace-nowrap font-medium text-slate-900 dark:border-slate-700 dark:text-slate-100"
            >{t.level}</td
          >
          <td
            class="border border-slate-200 px-4 py-3 align-top text-slate-700 dark:border-slate-700 dark:text-slate-300"
            >{@html t.description}</td
          >
        </tr>
      {/each}
    </tbody>
  </table>
</div>
