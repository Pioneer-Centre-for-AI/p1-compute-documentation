<script lang="ts">
  // An iframe whose content has a fixed width wider than the prose column,
  // like an Airtable kanban (fixed-width columns, no fit-to-width option).
  // It renders at its natural size and is scaled down to the container, so
  // every column shows at once. Below minScale the text gets too small to
  // read, so on a phone it falls back to its natural size and scrolls.
  type Props = {
    src: string;
    title: string;
    /** Width the content needs to show without scrolling, in CSS pixels. */
    width: number;
    /** Height to render at before scaling, in CSS pixels. */
    height: number;
    minScale?: number;
  };
  let { src, title, width, height, minScale = 0.55 }: Props = $props();

  let containerWidth = $state(0);
  // 0 until hydration measures the container: render unscaled until then.
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
