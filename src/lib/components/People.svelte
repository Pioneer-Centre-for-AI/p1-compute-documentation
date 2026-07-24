<script lang="ts">
  type Person = {
    name: string;
    role: string;
    institution: string;
    email: string;
    profileUrl?: string;
  };
  type Props = { people: Person[] };
  let { people }: Props = $props();

  const initials = (name: string) =>
    name
      .split(/\s+/)
      .filter(Boolean)
      .slice(0, 2)
      .map((part) => part[0]?.toUpperCase() ?? '')
      .join('');
</script>

<div class="not-prose grid grid-cols-1 gap-5 my-6 sm:grid-cols-2 lg:grid-cols-3">
  {#each people as person}
    <div
      class="group relative flex flex-col overflow-hidden rounded-2xl border border-slate-200/80 bg-white shadow-sm transition-all duration-200 hover:-translate-y-1 hover:border-slate-400 hover:shadow-xl dark:border-slate-700/70 dark:bg-slate-800/40 dark:hover:border-slate-500"
    >
      <span
        class="absolute inset-x-0 top-0 h-1 bg-slate-900 dark:bg-slate-100"
        aria-hidden="true"
      ></span>

      <div class="flex flex-col gap-4 p-5">
        <div class="flex items-center justify-between gap-3">
          <div
            class="flex h-12 w-12 shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-slate-800 to-slate-950 font-mono text-sm font-semibold tracking-wide text-white shadow-inner select-none dark:from-slate-600 dark:to-slate-800"
            aria-hidden="true"
          >
            {initials(person.name)}
          </div>
          <span
            class="eyebrow shrink-0 rounded-full bg-slate-100 px-2.5 py-1 !text-slate-600 dark:bg-slate-700/60 dark:!text-slate-300"
          >
            {person.role}
          </span>
        </div>

        <div>
          <h3 class="font-semibold leading-snug text-slate-900 dark:text-slate-100">
            {#if person.profileUrl}
              <a
                href={person.profileUrl}
                target="_blank"
                rel="noopener noreferrer"
                class="no-underline transition-colors hover:text-[var(--color-brand)]"
              >
                {person.name}
              </a>
            {:else}
              {person.name}
            {/if}
          </h3>
          <p class="mt-1 text-sm text-slate-600 dark:text-slate-400">{person.institution}</p>
        </div>
      </div>

      <a
        href="mailto:{person.email}"
        class="mt-auto flex items-center gap-1.5 border-t border-slate-100 px-5 py-3.5 text-sm font-medium text-slate-500 no-underline transition-colors hover:bg-slate-50 hover:text-[var(--color-brand)] dark:border-slate-700/60 dark:text-slate-400 dark:hover:bg-slate-700/30"
      >
        <svg class="h-4 w-4 shrink-0" viewBox="0 0 20 20" fill="none" aria-hidden="true">
          <rect x="2.5" y="4.5" width="15" height="11" rx="2" stroke="currentColor" stroke-width="1.5" />
          <path d="m3.5 6 6.5 4.5L16.5 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        <span class="truncate">{person.email}</span>
      </a>
    </div>
  {/each}
</div>
