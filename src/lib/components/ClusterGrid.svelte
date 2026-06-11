<script lang="ts">
  let { children }: { children: import('svelte').Snippet } = $props();
</script>

<div class="flex flex-col gap-12 my-8">
  {@render children()}
</div>
