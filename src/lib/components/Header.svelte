<script lang="ts">
  import ThemeToggle from './ThemeToggle.svelte';
</script>

<header
  class="sticky top-0 z-30 flex items-center justify-between border-b border-slate-200 bg-white/80 px-4 py-3 backdrop-blur md:px-6 dark:border-slate-700 dark:bg-slate-900/80"
>
  <a href="/" class="flex items-center gap-3 no-underline">
    <img src="/assets/logos/PCAI_Solo_RGB.svg" alt="" class="block h-8 w-8 dark:hidden" />
    <img src="/assets/logos/PCAI_Solo_White.svg" alt="" class="hidden h-8 w-8 dark:block" />
    <span class="text-sm font-semibold text-slate-900 sm:text-base dark:text-slate-100">
      P1 HPC Documentation
    </span>
  </a>
  <div class="flex items-center gap-2">
    <a
      href="https://github.com/Pioneer-Centre-for-AI/p1-compute-documentation"
      target="_blank"
      rel="noopener noreferrer"
      aria-label="GitHub"
      class="inline-flex h-9 w-9 items-center justify-center rounded-md border border-slate-200 text-slate-700 hover:bg-slate-100 dark:border-slate-700 dark:text-slate-200 dark:hover:bg-slate-800"
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
        <path
          d="M12 .5C5.65.5.5 5.65.5 12c0 5.08 3.29 9.39 7.86 10.91.58.11.79-.25.79-.56v-2.16c-3.2.7-3.87-1.36-3.87-1.36-.52-1.33-1.27-1.69-1.27-1.69-1.04-.71.08-.7.08-.7 1.15.08 1.76 1.18 1.76 1.18 1.02 1.75 2.68 1.25 3.34.96.1-.74.4-1.25.72-1.54-2.55-.29-5.24-1.28-5.24-5.71 0-1.26.45-2.29 1.18-3.1-.12-.29-.51-1.47.11-3.06 0 0 .97-.31 3.18 1.18a11 11 0 0 1 5.8 0c2.21-1.49 3.18-1.18 3.18-1.18.62 1.59.23 2.77.11 3.06.74.81 1.18 1.84 1.18 3.1 0 4.44-2.69 5.41-5.26 5.7.41.35.77 1.03.77 2.08v3.08c0 .31.21.68.8.56C20.21 21.39 23.5 17.08 23.5 12 23.5 5.65 18.35.5 12 .5z"
        />
      </svg>
    </a>
    <ThemeToggle />
  </div>
</header>
