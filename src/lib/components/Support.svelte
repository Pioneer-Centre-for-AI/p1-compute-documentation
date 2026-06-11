<script lang="ts">
  type Props = {
    showComputeCoordinator?: boolean;
    technicalSupportEmail?: string;
    technicalSupportLabel?: string;
    computeCoordinatorEmail?: string;
    emailSubjectPrefix?: string;
  };
  let {
    showComputeCoordinator = true,
    technicalSupportEmail = 'support@hpc-facility.dk',
    technicalSupportLabel = 'HPC Support Team',
    computeCoordinatorEmail = 'bstja@dtu.dk',
    emailSubjectPrefix = ''
  }: Props = $props();

  function mailto(email: string, subjectSuffix = ''): string {
    if (!emailSubjectPrefix && !subjectSuffix) return `mailto:${email}`;
    const subject = `${emailSubjectPrefix}${subjectSuffix ? ` ${subjectSuffix}` : ''}`.trim();
    return `mailto:${email}?subject=${encodeURIComponent(subject)}`;
  }
</script>

<div class="not-prose my-6 grid grid-cols-1 md:grid-cols-2 gap-6">
  <div class="rounded-xl border border-slate-200 bg-slate-50 p-6 dark:border-slate-700 dark:bg-slate-800/50">
    <div class="mb-3 flex items-center gap-3 text-blue-600 dark:text-blue-400">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/>
      </svg>
      <h3 class="m-0 text-lg font-semibold text-slate-900 dark:text-slate-100">Technical Support</h3>
    </div>
    <p class="text-slate-600 dark:text-slate-400">
      For technical issues, contact the
      <a href={mailto(technicalSupportEmail, 'Technical Issue')} class="text-blue-600 dark:text-blue-400 font-medium">{technicalSupportLabel}</a>.
    </p>
  </div>

  <div class="rounded-xl border border-slate-200 bg-slate-50 p-6 dark:border-slate-700 dark:bg-slate-800/50">
    <div class="mb-3 flex items-center gap-3 text-blue-600 dark:text-blue-400">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14,2 14,8 20,8"/>
      </svg>
      <h3 class="m-0 text-lg font-semibold text-slate-900 dark:text-slate-100">Policy Support</h3>
    </div>
    <p class="text-slate-600 dark:text-slate-400">
      For policy issues, contact
      <a href={mailto('compute-governance-p1@aicentre.dk', 'Policy Question')} class="text-blue-600 dark:text-blue-400 font-medium">compute-governance-p1@aicentre.dk</a>.
    </p>
  </div>

  <div class="rounded-xl border border-slate-200 bg-slate-50 p-6 dark:border-slate-700 dark:bg-slate-800/50">
    <div class="mb-3 flex items-center gap-3 text-blue-600 dark:text-blue-400">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
      </svg>
      <h3 class="m-0 text-lg font-semibold text-slate-900 dark:text-slate-100">General Questions</h3>
    </div>
    <p class="text-slate-600 dark:text-slate-400">
      Use the
      <a href="https://pioneercentreforai.slack.com/channels/compute" target="_blank" rel="noopener noreferrer" class="rounded bg-slate-200 dark:bg-slate-700 px-1 py-0.5 font-mono text-sm no-underline">#compute</a>
      or
      <a href="https://pioneercentreforai.slack.com/channels/ask-compute" target="_blank" rel="noopener noreferrer" class="rounded bg-slate-200 dark:bg-slate-700 px-1 py-0.5 font-mono text-sm no-underline">#ask-compute</a>
      channels on P1 Slack.
    </p>
  </div>

  {#if showComputeCoordinator}
    <div class="rounded-xl border border-slate-200 bg-slate-50 p-6 dark:border-slate-700 dark:bg-slate-800/50">
      <div class="mb-3 flex items-center gap-3 text-blue-600 dark:text-blue-400">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
          <circle cx="12" cy="7" r="4"/>
        </svg>
        <h3 class="m-0 text-lg font-semibold text-slate-900 dark:text-slate-100">Compute Coordinator</h3>
      </div>
      <p class="text-slate-600 dark:text-slate-400">
        Contact
        <a href={mailto(computeCoordinatorEmail, 'General Inquiry')} class="text-blue-600 dark:text-blue-400 font-medium">{computeCoordinatorEmail}</a>
        for general or technical compute-related questions.
      </p>
    </div>
  {/if}
</div>
