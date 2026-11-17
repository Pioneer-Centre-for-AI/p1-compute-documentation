<script lang="ts">
  import RfcStatus from '#lib/components/RfcStatus.svelte';
  import { formatDay, type RfcEntry } from '#lib/rfcs.ts';

  let { rfcs }: { rfcs: RfcEntry[] } = $props();
</script>

{#if rfcs.length === 0}
  <p class="not-prose text-sm text-slate-600 dark:text-slate-400">No RFC is open.</p>
{:else}
  <ul
    class="not-prose divide-y divide-slate-200 border-y border-l-2 border-y-slate-200 border-l-[var(--color-brand)] dark:divide-slate-800 dark:border-y-slate-800"
  >
    {#each rfcs as rfc}
      <li>
        <a
          href={rfc.href}
          class="group grid grid-cols-[1fr_auto] items-center gap-x-4 gap-y-1 px-4 py-3 no-underline transition-colors hover:bg-slate-50 focus-visible:bg-slate-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-slate-500 sm:grid-cols-[4.5rem_minmax(0,1fr)_auto_6rem] dark:hover:bg-slate-800/50 dark:focus-visible:bg-slate-800/50"
        >
          <span class="font-mono text-xs text-slate-600 dark:text-slate-400">RFC {rfc.number}</span>
          <span class="col-span-2 min-w-0 sm:col-span-1">
            <span
              class="block font-semibold text-slate-900 group-hover:text-[var(--color-brand)] dark:text-slate-100"
            >
              {rfc.meta.title}
            </span>
            <span class="block truncate text-sm text-slate-600 dark:text-slate-400">{rfc.meta.description}</span>
          </span>
          <span class="col-start-2 row-start-1 justify-self-end sm:col-start-auto sm:row-start-auto">
            <RfcStatus status={rfc.meta.status} />
          </span>
          <time
            datetime={rfc.meta.updated}
            class="hidden text-right text-xs tabular-nums text-slate-600 sm:block dark:text-slate-400"
          >
            {formatDay(rfc.meta.updated, 'short')}
          </time>
        </a>
      </li>
    {/each}
  </ul>
{/if}
