<script lang="ts">
  import EmbedForm from './EmbedForm.svelte';
  import { ACCESS_EXCEPTION_FORM_URL } from '$lib/links';
</script>

<p>
  <strong>PhD P1 affiliates and above can skip this section.</strong> If you are a P1 member
  currently enrolled in an MSc, a research assistant (RA) role, or similar, you are not eligible
  for access without a written approval of exception from a responsible person (a P1 co-lead or
  faculty member) who can vouch for your request. Fill in the form and have the responsible person
  send a confirmation to
  <a href="mailto:compute-governance-p1@aicentre.dk">compute-governance-p1@aicentre.dk</a> so they
  can audit the request. The exception applies across all P1 clusters.
</p>

<EmbedForm
  title="P1 HPC Access Exception Form"
  src="{ACCESS_EXCEPTION_FORM_URL}?embedded=true"
  href={ACCESS_EXCEPTION_FORM_URL}
  description="Requires sign-off from a P1 co-lead or faculty member."
/>
