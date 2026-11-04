<script lang="ts">
  type Props = {
    src: string;
    title: string;
    width: number;
    height: number;
    minScale?: number;
  };
  let { src, title, width, height, minScale = 0.55 }: Props = $props();

  let containerWidth = $state(0);
  const fits = $derived(containerWidth > 0 && containerWidth / width >= minScale);
  const scale = $derived(fits ? Math.min(1, containerWidth / width) : 1);
</script>

<div
  bind:clientWidth={containerWidth}
  class="overflow-y-hidden {fits ? 'overflow-x-hidden' : 'overflow-x-auto'}"
  style:height="{height * scale}px"
>
  <iframe
    {src}
    {title}
    loading="lazy"
    class="block origin-top-left border-0 bg-white"
    style:width="{Math.max(width, containerWidth)}px"
    style:height="{height}px"
    style:transform={scale < 1 ? `scale(${scale})` : undefined}
  ></iframe>
</div>
