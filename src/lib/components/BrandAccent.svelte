<script lang="ts">
  type Props = { color: string; width?: string };
  let { color, width = 'w-full mb-6' }: Props = $props();
</script>

<div
  aria-hidden="true"
  class="h-[3px] rounded-sm {width}"
  style:background-color={color}
></div>
