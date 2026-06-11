<script lang="ts">
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

<div class="flex gap-8 items-center py-4 max-[900px]:!flex-col max-[900px]:items-stretch" class:flex-row-reverse={reverse}>
  <div class="flex-1 relative rounded-lg overflow-hidden min-w-[320px] max-w-[520px] flex items-center justify-center">
    <img
      src={image}
      alt={imageAlt}
      loading="lazy"
      class="w-full h-auto aspect-video object-cover rounded-lg block"
    />
  </div>
  <div
    class="flex-1 p-4 flex flex-col justify-center max-[900px]:!items-start max-[900px]:!text-left"
    class:items-end={reverse}
    class:text-right={reverse}
    class:items-start={!reverse}
    class:text-left={!reverse}
  >
    <h2 class="text-3xl font-bold mb-4 text-slate-900 dark:text-slate-100">{title}</h2>
    <div class="mb-6 leading-relaxed text-slate-700 dark:text-slate-300">
      {@render children()}
    </div>
    <a
      href={learnMoreLink}
      class="mt-2 inline-flex items-center gap-2 rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white no-underline hover:bg-blue-700"
    >
      Learn More
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><line x1="5" y1="12" x2="19" y2="12"/><polyline points="12,5 19,12 12,19"/></svg>
    </a>
  </div>
</div>
