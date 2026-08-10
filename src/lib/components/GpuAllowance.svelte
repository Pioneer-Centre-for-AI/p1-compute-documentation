<script lang="ts">
  import EmbedForm from './EmbedForm.svelte';
  import { GPU_ALLOWANCE_URL } from '$lib/links';
</script>

<p>
  Per-user GPU limits keep the cluster fair when demand is high. When resources are
  underutilized, you can request a temporary increase of your GPU allowance. Increases are
  granted on a best-effort basis and roll back when demand picks up again.
</p>

<EmbedForm
  title="Temporary GPU Allowance Increase"
  src="{GPU_ALLOWANCE_URL}?embed=true"
  href={GPU_ALLOWANCE_URL}
  description="Tell us which cluster, how many GPUs, and for how long."
/>
