<script lang="ts">
  import Aside from '$lib/components/Aside.svelte';
  import Support from '$lib/components/Support.svelte';
</script>

<svelte:head>
  <title>Gefion HPC (Partner)</title>
</svelte:head>

<h1>Gefion HPC (Partner)</h1>

<Aside type="caution" title="Interest List | Vouchers Not Available At This Time">
  We are still in the process of figuring out how we can best support researchers who wish to use
  the Gefion HPC for their research. Until then, you are welcome to sign up for the interest list
  via the form below and tell us more about your use case and then we'll keep you updated on any
  progress.
</Aside>

<p>
  The Gefion HPC is a national-level facility that can support both Tier 2 and some Tier 1
  workloads. It is particularly suitable for large-scale model training and distributed computing
  tasks.
</p>

<h2>Playground Compute Vouchers</h2>

<p>
  P1 plans to allocate a portion of its compute budget to support researchers through "Gefion
  Playground" vouchers. These vouchers enable researchers to test their compatibility with the
  Gefion HPC system and explore the possibilities of large-scale computing.
</p>

<Aside type="note" title="Important">
  These vouchers are <strong>not intended for full-scale experiments</strong> — they are for trial
  usage, compatibility testing, and exploring large-scale computing capabilities. Once you've
  confirmed your setup works and evaluated the potential, you'll need to secure your own project
  funding for production workloads.
</Aside>

<p>
  Voucher signup is not yet open. Please use the form below if you'd like to be notified when
  vouchers become available.
</p>

<h2>Support</h2>

<Support
  technicalSupportEmail="hello@dcai.dk"
  technicalSupportLabel="DCAI Support Team"
  computeCoordinatorEmail="bstja@dtu.dk"
  emailSubjectPrefix="P1 Gefion HPC:"
/>

<h2>Specification</h2>

<ul>
  <li>NVIDIA DGX SuperPOD</li>
  <li>Multiple DGX nodes (8 × H100s) for a total of 1,528 H100 GPUs with high-performance interconnects</li>
  <li>Large-scale GPU resources for distributed training</li>
  <li>High-bandwidth storage solutions</li>
  <li>Scheduling Environment: SLURM</li>
</ul>
