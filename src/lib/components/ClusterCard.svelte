<script lang="ts">
  import Button from './Button.svelte';

  type Props = {
    title: string;
    image: string;
    imageAlt: string;
    learnMoreLink: string;
    reverse?: boolean;
    children: import('svelte').Snippet;
  };
  let { title, image, imageAlt, learnMoreLink, reverse = false, children }: Props = $props();
</script>

<div
  class="flex items-center gap-8 py-4 max-[900px]:!flex-col max-[900px]:items-stretch"
  class:flex-row-reverse={reverse}
>
  <div class="flex flex-1 min-w-[320px] max-w-[520px] items-center justify-center overflow-hidden rounded-lg">
    <img
      src={image}
      alt={imageAlt}
      loading="lazy"
      class="block aspect-video h-auto w-full rounded-lg object-cover"
    />
  </div>
  <div
    class="flex flex-1 flex-col justify-center p-4 max-[900px]:!items-start max-[900px]:!text-left"
    class:items-end={reverse}
    class:text-right={reverse}
    class:items-start={!reverse}
    class:text-left={!reverse}
  >
    <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-100">{title}</h2>
    <div class="mb-6 leading-relaxed text-slate-700 dark:text-slate-300">
      {@render children()}
    </div>
    <Button href={learnMoreLink} variant="primary">
      {#snippet children()}
        Learn More
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><line x1="5" y1="12" x2="19" y2="12"/><polyline points="12,5 19,12 12,19"/></svg>
      {/snippet}
    </Button>
  </div>
</div>
