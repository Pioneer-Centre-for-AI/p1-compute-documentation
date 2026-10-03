<script lang="ts">
  import { sessionTypes, sessionTypeColor, type ProgrammeRow } from '$lib/events';

  type Props = { rows: ProgrammeRow[] };
  let { rows }: Props = $props();

  // Times are "HH:MM-HH:MM" clock times, so the start is what the timeline
  // hangs off and the length is what makes the shape of the session readable
  // (a third talks, the rest discussion).
  const at = (range: string) => range.split('-')[0];
  const until = (range: string) => range.split('-')[1];
  const asMinutes = (clock: string) => {
    const [h, m] = clock.split(':').map(Number);
    return h * 60 + m;
  };
  const lengthOf = (range: string) => asMinutes(until(range)) - asMinutes(at(range));
</script>

<div class="not-prose mt-6 mb-8">
  <ul class="mb-6 flex flex-wrap gap-2">
    {#each sessionTypes as t}
      <li
        class="flex items-center gap-2 rounded-full border border-slate-200 px-3 py-1 text-xs text-slate-600 dark:border-slate-700 dark:text-slate-400"
      >
        <span
          class="inline-block h-2 w-2 shrink-0 rounded-full"
          style:background-color={t.color}
        ></span>
        {t.label}
      </li>
    {/each}
  </ul>

  <ol class="relative">
    {#each rows as row, i}
      <li class="grid grid-cols-[3.5rem_1.25rem_1fr] gap-x-3 sm:grid-cols-[5rem_1.5rem_1fr] sm:gap-x-4">
        <!-- Start time; the end is the next row's start, so only the length is
             repeated here. -->
        <div class="pt-3.5 text-end">
          <span
            class="block font-mono text-sm font-medium tabular-nums text-slate-900 dark:text-slate-100"
            >{at(row.time)}</span
          >
          <span class="block font-mono text-[11px] tabular-nums text-slate-400 dark:text-slate-500"
            >{lengthOf(row.time)} min</span
          >
        </div>

        <!-- Rail: one continuous line, a dot per session, coloured by type. -->
        <div class="relative flex justify-center" aria-hidden="true">
          <span
            class="absolute bottom-0 left-1/2 w-px -translate-x-1/2 bg-slate-200 dark:bg-slate-700"
            class:top-0={i > 0}
            class:top-5={i === 0}
          ></span>
          <span
            class="relative mt-[1.1rem] h-2.5 w-2.5 rounded-full ring-4 ring-white dark:ring-slate-900"
            style:background-color={sessionTypeColor[row.type]}
          ></span>
        </div>

        <div
          class="min-w-0 rounded-lg px-3 py-3 {row.type === 'interactive'
            ? 'bg-coral/5 dark:bg-coral/10'
            : ''}"
        >
          <p class="font-medium text-slate-900 dark:text-slate-100">{row.session}</p>
          <p class="mt-0.5 text-sm text-slate-500 dark:text-slate-400">
            {#if row.speakerUrl}
              <a
                href={row.speakerUrl}
                target="_blank"
                rel="noopener noreferrer"
                class="font-medium text-[var(--color-brand)] no-underline hover:underline"
                >{row.speaker}</a
              >
            {:else}
              <span class="font-medium text-slate-700 dark:text-slate-300">{row.speaker}</span>
            {/if}{#if row.affiliation}<span class="text-slate-400 dark:text-slate-500"
                >{" · " + row.affiliation}</span
              >{/if}{#if row.form}<span class="text-slate-400 dark:text-slate-500">{" · "}</span><a
                href={row.form.href}
                target="_blank"
                rel="noopener noreferrer"
                class="font-medium text-[var(--color-brand)] no-underline hover:underline"
                >{row.form.label}</a
              >{/if}
          </p>
        </div>
      </li>
    {/each}

    <!-- Terminus, so the last block reads as closed rather than cut off. -->
    <li class="grid grid-cols-[3.5rem_1.25rem_1fr] gap-x-3 sm:grid-cols-[5rem_1.5rem_1fr] sm:gap-x-4">
      <div class="text-end">
        <span class="block font-mono text-sm tabular-nums text-slate-400 dark:text-slate-500"
          >{until(rows[rows.length - 1].time)}</span
        >
      </div>
      <div class="relative flex justify-center" aria-hidden="true">
        <span
          class="absolute top-0 left-1/2 h-2 w-px -translate-x-1/2 bg-slate-200 dark:bg-slate-700"
        ></span>
        <span
          class="relative mt-1 h-2.5 w-2.5 rounded-full border border-slate-300 bg-white ring-4 ring-white dark:border-slate-600 dark:bg-slate-900 dark:ring-slate-900"
        ></span>
      </div>
      <p class="eyebrow pt-0.5 ps-3">End of session</p>
    </li>
  </ol>
</div>
