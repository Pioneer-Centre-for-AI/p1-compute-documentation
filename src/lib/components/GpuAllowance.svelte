<script lang="ts">
  import EmbedForm from './EmbedForm.svelte';
  import { GPU_ALLOWANCE_URL } from '$lib/links';

  // The copy is framed around protection rather than size. Where a cluster lets
  // you submit past the default limit yourself (Gefion), that path is faster
  // and this form would only duplicate it; what it cannot give you is a run
  // that survives the moment demand returns.
  //
  // selfServeDocumented is set by a cluster whose page already spells out its
  // own way past the default, so the generic "check whether your cluster allows
  // it" pointer is not printed directly underneath the answer.
  let { selfServeDocumented = false }: { selfServeDocumented?: boolean } = $props();
</script>

<p>
  Default per-user limits keep the cluster fair when demand is high. Use this form when a run
  needs more than the default <em>and</em> needs to be protected: a multi-day training, a
  deadline, or a workload that cannot checkpoint. Tell us the size and the window, and the
  compute coordinator arranges it with the cluster operator.
</p>

{#if !selfServeDocumented}
  <p>
    If your workload checkpoints cleanly and you only want GPUs that are sitting idle, check
    whether your cluster lets you submit past the default limit directly. That path is faster and
    needs no request, at the cost of the job being cancellable when demand picks up.
  </p>
{/if}

<EmbedForm
  title="Planned GPU Allocation"
  src="{GPU_ALLOWANCE_URL}?embed=true"
  href={GPU_ALLOWANCE_URL}
  description="Tell us which cluster, how many GPUs, for how long, and by when."
/>
