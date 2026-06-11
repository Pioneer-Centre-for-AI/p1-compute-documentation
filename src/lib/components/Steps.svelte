<script lang="ts">
  let { children }: { children: import('svelte').Snippet } = $props();
</script>

<ol class="my-6 list-none space-y-6 border-l border-slate-300 pl-6 dark:border-slate-700 [counter-reset:steps]">
  {@render children()}
</ol>

<style>
  ol :global(> li) {
    position: relative;
    counter-increment: steps;
  }

  ol :global(> li::before) {
    content: counter(steps);
    position: absolute;
    left: -2.5rem;
    top: 0;
    width: 1.75rem;
    height: 1.75rem;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.875rem;
    font-weight: 600;
    color: white;
    background: #2563eb;
    border-radius: 9999px;
  }
</style>
