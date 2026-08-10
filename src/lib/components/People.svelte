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

<ul
  class="not-prose my-6 divide-y divide-slate-200 overflow-hidden rounded-xl border border-slate-200 bg-slate-50 dark:divide-slate-700 dark:border-slate-700 dark:bg-slate-800/40"
>
  {#each people as person}
    <!-- Wraps below sm: the email is long enough in mono that keeping it on the
         same row squeezes the name down to a few characters on a phone. -->
    <li class="flex flex-wrap items-center gap-x-4 gap-y-1 px-4 py-3">
      <span
        class="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-slate-200 font-mono text-xs font-semibold text-slate-700 select-none dark:bg-slate-700 dark:text-slate-200"
        aria-hidden="true"
      >
        {initials(person.name)}
      </span>
      <span class="flex min-w-0 flex-1 flex-col gap-x-3 sm:flex-row sm:items-center">
        <span class="truncate font-medium text-slate-900 dark:text-slate-100">
          {#if person.profileUrl}
            <a
              href={person.profileUrl}
              target="_blank"
              rel="noopener noreferrer"
              class="no-underline hover:underline"
            >
              {person.name}
            </a>
          {:else}
            {person.name}
          {/if}
        </span>
        <span class="eyebrow shrink-0" title={person.institution}>{person.role}</span>
      </span>
      <a
        href="mailto:{person.email}"
        class="w-full shrink-0 ps-[3.25rem] font-mono text-[13px] break-all text-slate-600 no-underline hover:text-slate-900 hover:underline sm:w-auto sm:ps-0 dark:text-slate-400 dark:hover:text-slate-100"
      >
        {person.email}
      </a>
    </li>
  {/each}
</ul>
