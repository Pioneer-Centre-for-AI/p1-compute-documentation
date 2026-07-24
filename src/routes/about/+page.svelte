<script lang="ts">
  import People from '$lib/components/People.svelte';
  import NeedsCallout from '$lib/components/NeedsCallout.svelte';
  import { headingAnchors } from '$lib/actions/headingAnchors';

  const board = [
    {
      name: 'Nicki Skafte Detlefsen',
      role: 'DTU',
      institution: 'Technical University of Denmark',
      email: 'nsde@dtu.dk',
      profileUrl: 'https://orbit.dtu.dk/en/persons/nicki-skafte-detlefsen'
    },
    {
      name: 'Rob van der Goot',
      role: 'ITU',
      institution: 'IT University of Copenhagen',
      email: 'robv@itu.dk',
      profileUrl: 'https://pure.itu.dk/en/persons/rob-van-der-goot'
    },
    {
      name: 'Mikkel Fruelund Odgaard',
      role: 'KU',
      institution: 'University of Copenhagen',
      email: 'miod@di.ku.dk',
      profileUrl: 'https://di.ku.dk/ansatte/?pure=da/persons/770606'
    },
    {
      name: 'Andreas Aakerberg',
      role: 'AAU',
      institution: 'Aalborg University',
      email: 'anaa@create.aau.dk',
      profileUrl: 'https://vbn.aau.dk/da/persons/132677'
    },
    {
      name: 'Akhil Arora',
      role: 'AU',
      institution: 'Aarhus University',
      email: 'akhil.arora@cs.au.dk',
      profileUrl: 'https://www.au.dk/en/akhil.arora@cs.au.dk'
    }
  ];
</script>

<svelte:head>
  <title>About P1 Compute Resources</title>
  <meta name="description" content="Learn more about the P1AI's HPC resources." />
</svelte:head>

<article class="prose mx-auto w-full max-w-3xl" use:headingAnchors>

<p class="eyebrow not-prose">Pioneer Centre for AI</p>

<h1>About P1 Compute Resources</h1>

<p>
  The <strong>Pioneer Centre for Artificial Intelligence</strong> (P1) is a Danish cross-university
  research centre and consortium dedicated to advancing AI research through collaboration with
  partnering institutions.
</p>

<p>
  This documentation aims to centralize practical knowledge and provide a concise guide for P1
  affiliates on how to access and use our available high-performance computing (HPC) facilities.
</p>

<NeedsCallout />

<img src="/assets/PCAI_HQ.avif" alt="Pioneer Centre for AI headquarters" class="my-6 w-full rounded-lg" loading="lazy" />

<p>
  A central mission of P1 is to foster broad research collaboration, both within the centre and
  across its affiliated universities. Our HPC resources are designed to support P1 researchers at
  partner institutions, enabling projects of varying scale and complexity.
</p>

<p>
  For more information, visit the
  <a href="https://www.aicentre.dk/" target="_blank" rel="noopener noreferrer">P1 Homepage</a> or
  <a href="https://www.aicentre.dk/computing" target="_blank" rel="noopener noreferrer">P1 Computing</a>.
</p>

<h2>Available Compute Resources</h2>

<p>
  While we prioritize smaller projects as they represent the majority of research needs, we also
  support larger projects through our tiered compute resources, including the capability to train
  very large models and support European-level workloads on Tier 2/1 facilities. But, very large
  scale projects might be better supported within their own funding.
</p>

<p>
  In practical terms this means that while training a large language or vision model from scratch
  might not be suitable for our Tier 3 facilities, such projects can be accommodated on a Tier 2
  national facility (ex. Gefion HPC), which is specifically designed for large-scale model training
  and can even support some European-level (Tier 1) workloads.
</p>

<h2>Support</h2>

<p>
  In general the P1 does not have official support staff. Instead we rely on the community to help
  each other. However, for questions regarding the cluster, we have the following two contact
  e-mails:
</p>

<ul>
  <li>For technical issues, like broken or missing packages, contact <a href="mailto:support@hpc.dtu.dk">support@hpc.dtu.dk</a></li>
  <li>For policy issues, like priority access, queuing system etc. contact <a href="mailto:compute-governance-p1@aicentre.dk">compute-governance-p1@aicentre.dk</a></li>
</ul>

<p>
  If your submission to join the cluster has been pending for more than a week, you can contact the
  member of the P1 HPC governance board of your institution. General announcements regarding the
  cluster will be made in the
  <a href="https://pioneercentreforai.slack.com/channels/compute" target="_blank" rel="noopener noreferrer">#compute</a>
  channel on the official P1-slack. Similarly if you have general questions regarding compute,
  including using the P1-cluster, feel free to post those questions in the
  <a href="https://pioneercentreforai.slack.com/channels/ask-compute" target="_blank" rel="noopener noreferrer">#ask-compute</a>
  channel.
</p>

<h2>HPC Governance Board</h2>

<People people={board} />

<h2>Investing Your Compute Budget</h2>

<p>
  Generally, P1 can help purchasing hardware for you if it is to be a part of P1's clusters. In
  this setup you would have priority access to the hardware that you purchase, but if it is unused
  other researchers at P1 can use it as well. Most HPC hardware has an expected utilization time of
  approximately 5 years.
</p>

<p>An example of a compute budget expenditure:</p>

<blockquote>
  A researcher might have 400k DKK for non-GDPR computing. In that case P1 can purchase a 2xA100
  node, with the researcher getting priority access. With the last 100k DKK, we recommend a
  diversification of the researchers compute portfolio.
</blockquote>

<blockquote>
  Using some of the 100k for personal computers with powerful GPU's and reserving a significant
  portion of funds for later. This could be for storage or cloud compute. Having the budget to
  spring for cloud compute if peak pressure on other computing resources makes training a model in
  time for a submission deadline infeasible can be necessary.
</blockquote>

<h2>Hierarchy of Compute Resources</h2>

<p>
  Depending on what you are doing, you will need different levels of compute. The following table
  gives an overview of the different levels of compute and what they are suitable for.
</p>

<table>
  <thead>
    <tr>
      <th>Tier</th>
      <th>Organization level</th>
      <th>Description</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>4</td>
      <td>Personal</td>
      <td>
        Personal laptop and/or desktop with either no GPUs or a single one. At this level compute
        is limited and mostly consists of code development and running experiments with small
        datasets and models.
      </td>
    </tr>
    <tr>
      <td>3</td>
      <td>Institute</td>
      <td>
        The next level is compute clusters containing either several smaller GPUs or a few larger
        ones. Multiple experiments can be run in parallel and models can be parallelized over a few
        GPUs, but it is not feasible to train very large models from scratch. Expect limitations to
        wall time (a couple of days), storage (around ½–1 TB) and number of nodes/GPUs used (around
        2–16).
      </td>
    </tr>
    <tr>
      <td>2</td>
      <td>National</td>
      <td>
        National HPC facilities provide the necessary resources for large-scale model training.
        Expect fewer restrictions to wall time (1–2 weeks), storage (around 1–10 TB) and number of
        nodes/GPUs used (around 16–64). Some national facilities can also support European-level
        (Tier 1) workloads. For example, P1's Gefion HPC is a national facility that can support
        both Tier 2 and possibly Tier 1 workloads.
      </td>
    </tr>
    <tr>
      <td>1</td>
      <td>European</td>
      <td>
        European HPC centers offer significantly larger scale compute resources, typically providing
        easy access to 8–16 GPUs with the possibility of scaling up to 500–800 GPUs depending on
        project requirements and the specific HPC center. Unlike lower tiers, walltime is usually
        not restricted; instead, limitations are set on total GPU hours used. See
        <a href="https://eurohpc-ju.europa.eu/supercomputers/our-supercomputers_en#karolina" target="_blank" rel="noopener noreferrer">Karolina</a>
        and
        <a href="https://eurohpc-ju.europa.eu/supercomputers/our-supercomputers_en#meluxina" target="_blank" rel="noopener noreferrer">Meluxina</a>
        for examples of European HPC centers with GPUs.
      </td>
    </tr>
    <tr>
      <td>0</td>
      <td>Global</td>
      <td>
        The highest level of compute. Possibility to use above 1000 GPUs and perform calculations at
        exascale. Usually only available to large research projects and collaborations with
        industry. See
        <a href="https://eurohpc-ju.europa.eu/supercomputers/our-supercomputers_en#lumi" target="_blank" rel="noopener noreferrer">LUMI</a>
        for an example of a global HPC center with GPUs.
      </td>
    </tr>
  </tbody>
</table>

<p>
  The P1 DTU and NGC HPCs sit at level 3 in the hierarchy, while Gefion HPC primarily operates at
  level 2 but can support some level 1 workloads, making it capable of supporting both national and
  some European-level research projects.
</p>

</article>
