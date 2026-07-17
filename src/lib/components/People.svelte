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
</script>

<div class="not-prose grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3 my-6">
  {#each people as person}
    <div class="rounded-xl border border-slate-200 bg-slate-50 p-6 transition-all hover:-translate-y-0.5 hover:shadow-lg hover:border-slate-400 dark:border-slate-700 dark:bg-slate-800/50 dark:hover:border-slate-500">
      <div class="text-center">
        <h3 class="mb-2 font-semibold text-slate-900 dark:text-slate-100">
          {#if person.profileUrl}
            <a
              href={person.profileUrl}
              target="_blank"
              rel="noopener noreferrer"
              class="hover:text-[var(--color-brand)]"
            >
              {person.name}
            </a>
          {:else}
            {person.name}
          {/if}
        </h3>
        <p class="eyebrow mb-1 !text-[var(--color-plum)] dark:!text-[var(--color-plum-bright)]">{person.role}</p>
        <p class="mb-3 text-sm text-slate-600 dark:text-slate-400">{person.institution}</p>
        <a
          href="mailto:{person.email}"
          class="text-sm font-medium text-[var(--color-brand)] no-underline hover:underline"
        >
          {person.email}
        </a>
      </div>
    </div>
  {/each}
</div>
