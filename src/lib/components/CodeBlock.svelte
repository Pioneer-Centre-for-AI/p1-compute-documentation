<script lang="ts">
  import { highlight } from '$lib/shiki';

  type Props = { code: string; lang?: string };
  let { code, lang = 'text' }: Props = $props();
  const html = $derived(highlight(code, lang));
</script>

{#await html then rendered}
  <div class="my-4 overflow-x-auto rounded-lg text-sm">
    {@html rendered}
  </div>
{/await}
