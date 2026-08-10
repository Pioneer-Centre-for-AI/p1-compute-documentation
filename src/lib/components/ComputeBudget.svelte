<script lang="ts">
  import EmbedForm from './EmbedForm.svelte';
  import { COMPUTE_BUDGET_URL } from '$lib/links';
</script>

<p>
  Planning a purchase is easier before the money is committed. Tell the compute coordinator what
  you have, what you plan to run, and how long it has to last, and you will get a recommendation
  on the split between cluster hardware, storage, cloud, and workstations.
</p>

<EmbedForm
  title="Investing Your Compute Budget"
  src="{COMPUTE_BUDGET_URL}?embed=true"
  href={COMPUTE_BUDGET_URL}
  description="Budget, workload, and the portfolio split. Nothing here commits you."
/>
