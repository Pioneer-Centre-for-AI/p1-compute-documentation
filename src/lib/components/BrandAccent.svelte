<script lang="ts">
  type Props = { color: string };
  let { color }: Props = $props();
</script>

<div
  aria-hidden="true"
  class="h-[3px] w-full rounded-sm mb-6"
  style:background-color={color}
></div>
