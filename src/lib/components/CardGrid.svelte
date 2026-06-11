<script lang="ts">
  let { children }: { children: import('svelte').Snippet } = $props();
</script>

<div class="grid grid-cols-1 gap-6 md:grid-cols-2">
  {@render children()}
</div>
