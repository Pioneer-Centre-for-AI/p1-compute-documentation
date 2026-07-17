<script lang="ts">
  import Button from './Button.svelte';

  type Props = { label?: string };
  let { label = 'Apply for P1 affiliation' }: Props = $props();
</script>

<div class="not-prose my-4">
  <Button
    href="https://www.aicentre.dk/p1adb"
    variant="primary"
    target="_blank"
    rel="noopener noreferrer"
  >
    {#snippet children()}
      {label}
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M15 3h6v6M10 14L21 3M21 14v7H3V3h7" />
      </svg>
    {/snippet}
  </Button>
</div>
