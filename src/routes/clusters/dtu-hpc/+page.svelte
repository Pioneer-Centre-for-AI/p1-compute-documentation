<script lang="ts">
  import Aside from '$lib/components/Aside.svelte';
  import CodeBlock from '$lib/components/CodeBlock.svelte';
  import EmbedForm from '$lib/components/EmbedForm.svelte';
  import Steps from '$lib/components/Steps.svelte';
  import Support from '$lib/components/Support.svelte';
</script>

<svelte:head>
  <title>P1 DTU HPC</title>
</svelte:head>

<h1>P1 DTU HPC</h1>

<p>
  The P1 DTU HPC is hosted at DTU and provides high-performance computing resources for P1 members
  (PhD and above). It is particularly suitable for medium to large-scale machine learning
  experiments and research projects.
</p>

<h2>Getting Access</h2>

<p>
  <strong>Requirements:</strong> PhD or higher (exceptions may apply), valid Danish university
  email, and registered P1 affiliation.
</p>

<Steps>
  <li>
    <EmbedForm
      title="P1 Affiliation Form"
      src="https://da.surveymonkey.com/r/P1Affiliation"
      minHeight="1200px"
    />
    Before accessing the P1 DTU HPC, you must first
    <a href="https://www.aicentre.dk/affiliation" target="_blank" rel="noopener noreferrer">register to become a member of P1</a>.
  </li>
  <li>
    <EmbedForm
      title="P1 HPC Access Exception Form"
      src="https://docs.google.com/forms/d/e/1FAIpQLSciaOm-CYwl48LGGZC7qlmCSPU7kfEitjGy4kvZXAXEPbo_eA/viewform?embedded=true"
      minHeight="1200px"
    />
    <strong>PhD P1 affiliates and above can skip this step.</strong> If you are a P1 member currently
    enrolled in a MSc, research assistant (RA) role, or similar, then you are not eligible for access
    without a written approval of exception from a responsible person (a P1 co-lead or faculty member)
    who can vouch for your request. Please fill in the following form and have the responsible person
    send a confirmation to the governance mail at
    <a href="mailto:compute-governance-p1@aicentre.dk">compute-governance-p1@aicentre.dk</a> so they
    can audit the request.
  </li>
  <li>
    <EmbedForm
      title="DTU Account Signup Form"
      src="https://forms.office.com/pages/responsepage.aspx?id=I_FR8s7JjkSSdzS7KFkR2TcQSfcccxdDtaq8HEbAO1RUQjZUUEoxSlRCRk5KV09WSlRZNjNLVE1KRi4u"
      height="800px"
    />
    Fill out the DTU account request form. Only signup using an official university email address is
    accepted. The form will be processed by Henning Christiansen, head of DTU's compute center and
    you will receive account details via email once created.
  </li>
</Steps>

<h2>Connecting to the Cluster</h2>

<p>The compute cluster is accessible at <code>login9.hpc.dtu.dk</code> via SSH. Note that:</p>

<ul>
  <li>Home directories have limited storage (30GB)</li>
  <li>Additional storage is available at <code>/dtu/p1/</code></li>
  <li>Interactive node is available for package installation and test runs</li>
  <li>Heavy jobs should be submitted as batch jobs</li>
</ul>

<h3>Access from Outside DTU Network</h3>

<Steps>
  <li>Download Cisco AnyConnect VPN client (see OpenConnect for Linux)</li>
  <li>Go to https://dtubasen.dtu.dk and sign in via Azure multi-factor auth using your full DTU username</li>
  <li>Set up multi-factor authentication</li>
  <li>Connect to vpn.dtu.dk using AnyConnect</li>
  <li>SSH to <code>login9.hpc.dtu.dk</code> using your DTU credentials</li>
</Steps>

<p>For persistent access, you can set up SSH keys:</p>

<Steps>
  <li>
    Generate key
    <CodeBlock lang="bash" code="ssh-keygen -t ed25519 -f ~/.ssh/keyname" />
  </li>
  <li>
    Copy public key
    <CodeBlock lang="bash" code="ssh-copy-id -i ~/.ssh/keyname.pub username@login9.hpc.dtu.dk" />
  </li>
  <li>
    Connect
    <CodeBlock lang="bash" code="ssh -i ~/.ssh/keyname username@login9.hpc.dtu.dk" />
  </li>
</Steps>

<p>
  TIP: Consider setting up a SSH host alias for <code>login9.hpc.dtu.dk</code> in your
  <code>~/.ssh/config</code> file to make it easier to connect to the cluster.
</p>

<h2>Support</h2>

<Support
  technicalSupportEmail="support@hpc.dtu.dk"
  technicalSupportLabel="DTU HPC Support Team"
  computeCoordinatorEmail="bstja@dtu.dk"
  emailSubjectPrefix="P1 DTU HPC:"
/>

<p>
  For more technical information, refer to the
  <a href="https://www.hpc.dtu.dk/?page_id=5028" target="_blank" rel="noopener noreferrer">P1 compute cluster documentation</a>
  at DTU DCC.
</p>

<h2>Fair Use Policy</h2>

<p>The following rules are in place to ensure fair use of the P1 DTU HPC:</p>

<ul>
  <li>Maximum wall time: 72 hours</li>
  <li>Maximum number of GPUs in a job: 2 (one node)</li>
  <li>Maximum running jobs: ~50% of total available GPUs</li>
  <li>500 gb of storage (+30gb in home directory)</li>
</ul>

<p>
  If you have a project that requires more storage resources than the above, please contact the
  governance group <a href="mailto:compute-governance-p1@aicentre.dk">compute-governance-p1@aicentre.dk</a>
  to discuss your needs.
</p>

<Aside type="caution" title="GDPR Compliance">
  The P1 DTU HPC is only intended for non-GDPR data e.g. public datasets, open benchmarks, etc. To
  be more specific, you can still work on private datasets because your home directory is not
  shared with other users, but data is not encrypted in a way to be GDPR compliant. For
  GDPR-compliant data processing, please refer to the P1 NGC HPC.
</Aside>

<h2>Specification</h2>

<ul>
  <li>7 Lenovo ThinkSystem SR665 V3 servers</li>
  <li>
    Each node specifications:
    <ul>
      <li>2 AMD EPYC 9354 32-Core Processors</li>
      <li>768GB RAM</li>
      <li>2 NVIDIA H100 PCIe GPUs (80GB each)</li>
    </ul>
  </li>
  <li>Storage: 60TiB shared storage</li>
  <li>Operating System: Alma Linux</li>
  <li>Scheduling Environment: LSF</li>
  <li>
    Resource Allocation:
    <ul>
      <li>7 nodes available for batch jobs (queue: <code>p1</code>)</li>
      <li>1 node reserved for interactive usage (queue: <code>p1i</code>)</li>
    </ul>
  </li>
</ul>
