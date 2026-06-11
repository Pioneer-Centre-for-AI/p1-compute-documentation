<script lang="ts">
  import Aside from '$lib/components/Aside.svelte';
  import CodeBlock from '$lib/components/CodeBlock.svelte';
  import EmbedForm from '$lib/components/EmbedForm.svelte';
  import Steps from '$lib/components/Steps.svelte';
  import Support from '$lib/components/Support.svelte';
</script>

<svelte:head>
  <title>P1 NGC HPC</title>
</svelte:head>

<h1>P1 NGC HPC</h1>

<Aside type="caution" title="Waiting List">
  We are currently in the process of tackling some requirements together with UCPH related to the
  setup of data processing agreements. We are not onboarding new users until this is in place. You
  can still sign up now to be placed on the waiting list, and we'll keep you updated on the
  progress.
</Aside>

<p>
  The P1 NGC HPC is hosted at the National Genome Centre and is designed for secure data processing
  with GDPR compliance. It provides a secure environment for handling sensitive data and research
  projects.
</p>

<h2>Getting Access</h2>

<p>
  <strong>Requirements:</strong> PhD or higher (exceptions may apply), valid Danish university
  email, and registered P1 affiliation.
</p>

<p>
  Each project needs to bring a record (a signed Data Processing Agreement should do) that
  explicitly mentions NGC as a data processor and that the data is allowed to be stored there. If
  the project poses a high risk to individuals whose personal data is being processed a Data
  Protection Impact Assessment (DPIA) will be needed also.
</p>

<Steps>
  <li>
    <EmbedForm
      title="P1 Affiliation Form"
      src="https://da.surveymonkey.com/r/P1Affiliation"
      minHeight="1200px"
    />
    Before accessing the P1 NGC HPC, you must first
    <a href="https://www.aicentre.dk/affiliation" target="_blank" rel="noopener noreferrer">register to become a member of P1</a>.
  </li>
  <li>
    Complete and sign the
    <a href="https://www.ngc.dk/Media/638139612542456428/Oprettelse%20af%20Ekstern%20Bruger%20p%C3%A5%20NGC%20Infrastruktur%200.4.pdf" target="_blank" rel="noopener noreferrer">NGC user creation form</a>
    and forward it to the
    <a href="mailto:bstja@dtu.dk">Compute Coordinator</a>
    to request access.
  </li>
</Steps>

<p>You will be added to the NGC slack channel once you gain access.</p>

<h3>Accessing the Cluster</h3>

<p>The P1 NGC HPC is an air-gapped system requiring:</p>

<ul>
  <li>Multi-factor authentication (MFA)</li>
  <li>A client for accessing the remote VM entrypoint</li>
  <li>
    Specific access instructions will follow after registration. But you can expect to use SFTP for
    transferring data into the system.
  </li>
</ul>

<p>
  Once connected with the Omnissa Remote Desktop client you can access the login node using
  <code>ssh -X &lt;your-username&gt;@login</code>.
</p>

<CodeBlock
  lang="text"
  code={`Important hosts:
    - https://console.cld076.vmc/status # Internal Status page
    - cld076-0004.cld076.vmc            # Internal (SFTP)
    - sftp.spc.ngc.dk                   # External Ingress/Egress (SFTP)`}
/>

<h2>Support</h2>

<Support
  technicalSupportEmail="support@ngc.dk"
  technicalSupportLabel="NGC HPC Support Team"
  computeCoordinatorEmail="bstja@dtu.dk"
  emailSubjectPrefix="P1 NGC HPC:"
/>

<h2>Specification</h2>

<ul>
  <li>Air-gapped system for secure data processing</li>
  <li>GDPR compliant infrastructure</li>
  <li>Secure storage solutions</li>
  <li>Specific hardware details available upon access approval</li>
  <li>Scheduling Environment: SLURM</li>
  <li>Resource allocation details provided during onboarding</li>
</ul>

<h2>FAQ (How-To)</h2>

<h3>How to transfer Sensitive &amp; Large Data?</h3>

<p>
  You can transfer sensitive and large data to the cluster using SFTP under the supervision of an
  admin. You will need to request access to the <code>/data/upload</code> directory as this acts as
  a data gateway.
</p>

<p>Then it's recommended to set up a SSH entry in your <code>~/.ssh/config</code>.</p>

<CodeBlock
  lang="text"
  code={`Host ngc
    HostName sftp.spc.ngc.dk
    Port 6433
    User <your-username>_sftp
    HostKeyAlgorithms +ssh-rsa`}
/>

<p>
  From here you can connect with <code>sftp ngc</code> and then <code>put</code> to the
  <code>/data/upload</code> directory from the outside and use <code>get</code> from the inside.
  During the transfer period the data is only accessible to you and the admins.
</p>

<p>
  As an alternative to using SFTP you can use <code>scp</code> to transfer the data to the cluster
  which is arguably easier:
</p>

<CodeBlock lang="bash" code="scp ~/datasets/ISLES-2022.zip ngc:/data/upload/" />

<p>Then inside the cluster you can transfer the data to your home directory using:</p>

<CodeBlock
  lang="bash"
  code="scp <your-username>_sftp@cld076-0004.cld076.vmc:/data/upload/ISLES-2022.zip ~/datasets/"
/>

<h3>How to transfer Miscellaneous Data?</h3>

<p>For miscellaneous and small data such as personal dotfiles or source code you can:</p>

<ul>
  <li>Transfer via SFTP (tedious and requires admin supervision)</li>
  <li>Use the internal server running a simple GitHub proxy/tunnel for public repositories.</li>
  <li>Mount a host directory using the Omnissa Remote Desktop client (must be enabled by an NGC admin)</li>
  <li>SSH into the admin node (if you have access to it)</li>
</ul>

<p>
  Your host clipboard works into the remote desktop client but not the other way around. You can
  take screenshots of the remote display during the session.
</p>

<h3>How to download git repos?</h3>

<p>Navigate to https://cld076-0006.cld076.vmc on the internal network.</p>

<h3>How to link internal Conda channel?</h3>

<CodeBlock
  lang="bash"
  code={`conda config --add channels https://vmc-nexus-01.ngc.vmc/repository/ngc-cloud-conda/main
conda config --remove channels defaults
conda config --set ssl_verify /etc/pki/ca-trust/source/anchors/ngc-cloud-root-ca.pem`}
/>

<h3>How to link internal Poetry channel?</h3>

<CodeBlock
  lang="bash"
  code={`export VENV_PATH=MYPATH
python3 -m venv $VENV_PATH

$VENV_PATH/bin/pip install -U pip setuptools
$VENV_PATH/bin/pip install poetry

$VENV_PATH/bin/poetry source add --priority=primary NGC https://vmc-nexus-01-ngc.vmc/repository/ngc-coud-pypi/simple
$VENV_PATH/bin/poetry config certificates.NGC.cert false`}
/>

<h3>How to start an interactive session?</h3>

<p>In MobaXterm you can start an interactive session by using <code>iqsub</code>.</p>

<CodeBlock
  lang="bash"
  code={`# Quick'n dirty interactive session
iqsub

# example: interactive session for a user in group named ngc-bio, 1 node and 4
# CPUs, 20 GB memory and running for 2 hours
qsub -I -X -W group_list=ngc-bio -A ngc-bio -l nodes=1:ppn=4,mem=20gb,walltime=02:00:00`}
/>

<h3>How to see my user information?</h3>

<p>To see info about your user, including the department/group(s) you belong to.</p>

<CodeBlock lang="bash" code="id" />

<h3>How to see available nodes?</h3>

<CodeBlock
  lang="bash"
  code={`# List info about available nodes
pbsnodes`}
/>

<h3>How to search for free resources?</h3>

<CodeBlock
  lang="bash"
  code={`# fat nodes
showbf -f fatnode
# thin nodes
showbf -f thinnode`}
/>

<h3>How to check resources used by a job?</h3>

<CodeBlock
  lang="bash"
  code={`# Get all jobs
qstat -a

# Get extended status of a specific job
qstat -f <jobid>

# Using a helper script
checkjob -v <jobid>`}
/>

<h3>How to PBS?</h3>

<CodeBlock
  lang="bash"
  code={`# get the status of your submitted jobs (Q: in queue but not yet running; R: running, C: complete)
qstat
# get extended status of your submitted jobs like requested resources and elapsed time
qstat -a
# see the nodes allocated to each run
qstat -n
# get the status of a specific job
qstat <jobid>
# more info about a specific job
qstat -f <jobid>
# view the qsub queue, including qstat all users
showq
# view running jobs (+ give additional info, including an estimate on how efficiently they are using the CPUs)
showq -r
# this will give you more information about your eligible jobs
showq -i
# for more options with the showq command:
showq -h
# if you want to check the status of a particular jobid
checkjob <jobid>
# add the -v flag to increase the verbosity
checkjob -v <jobid>
# check when a job will run
showstart <jobid>
# check resource usage of completed job (privileged command)
tracejob -v <jobid>
# to cancel a job, first find the job-id using qstat, then type:
qdel <jobid>
# to cancel a job using mjobclt, first find the job-id using qstat, then type:
mjobctl -c <jobid>`}
/>
