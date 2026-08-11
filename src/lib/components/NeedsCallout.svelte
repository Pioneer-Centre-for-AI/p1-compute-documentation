<script lang="ts">
  import { SURVEY_PAGE } from '$lib/links';
</script>

<aside
  class="not-prose my-6 overflow-hidden rounded-xl border border-slate-200 bg-slate-50 dark:border-slate-700 dark:bg-slate-800/40"
>
  <div class="flex flex-col gap-4 p-5 sm:flex-row sm:items-start">
    <div
      class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-[var(--color-coral)]/12 text-[var(--color-coral-strong)] dark:bg-[var(--color-coral)]/20 dark:text-[var(--color-coral)]"
      aria-hidden="true"
    >
      <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
        <path d="M12 2v4M12 18v4M2 12h4M18 12h4M5 5l2.5 2.5M16.5 16.5 19 19M19 5l-2.5 2.5M7.5 16.5 5 19" />
        <circle cx="12" cy="12" r="3.2" />
      </svg>
    </div>

    <div class="flex flex-col gap-2.5">
      <div>
        <p class="eyebrow">Help shape P1 compute</p>
        <h3 class="mt-1 text-lg font-semibold text-slate-900 dark:text-slate-100">
          What do you need: hardware or software?
        </h3>
      </div>
      <p class="text-sm leading-relaxed text-slate-600 dark:text-slate-400">
        We want to hear what researchers actually need from HPC, whether that is hardware (GPUs,
        memory, storage, interconnect) or the software layers you run on top. Fill in the short
        <strong class="font-semibold text-slate-700 dark:text-slate-300">Your Compute Needs</strong>
        survey to inform the compute coordinator, so we can take strategic action based on real data
        points instead of guesswork.
      </p>
      <a
        href={SURVEY_PAGE}
        class="inline-flex w-fit items-center gap-2 rounded-full bg-slate-800 px-4 py-2 text-sm font-medium text-white no-underline transition-colors hover:bg-slate-700 dark:bg-slate-200 dark:text-slate-900 dark:hover:bg-slate-300"
      >
        Share Your Compute Needs
        <svg class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <line x1="5" y1="12" x2="19" y2="12" />
          <polyline points="12,5 19,12 12,19" />
        </svg>
      </a>
    </div>
  </div>
</aside>
