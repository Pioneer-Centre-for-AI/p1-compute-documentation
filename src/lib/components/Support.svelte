<script lang="ts">
  import { SURVEY_URL, COORDINATOR_ORCID } from '$lib/links';

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

<dl
  class="not-prose my-6 divide-y divide-slate-200 overflow-hidden rounded-xl border border-slate-200 bg-slate-50 dark:divide-slate-700 dark:border-slate-700 dark:bg-slate-800/40"
>
  <div class="grid grid-cols-[2.5rem_1fr] items-center gap-4 p-4 sm:grid-cols-[2.5rem_1fr_auto] sm:gap-6">
    <div class="flex h-10 w-10 items-center justify-center rounded-full bg-slate-200 text-slate-700 dark:bg-slate-700 dark:text-slate-200">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/>
      </svg>
    </div>
    <div class="col-span-1">
      <dt class="font-semibold text-slate-900 dark:text-slate-100">Technical Support</dt>
      <dd class="text-sm text-slate-600 dark:text-slate-400">Broken or missing packages, scheduler issues</dd>
    </div>
    <a
      href={mailto(technicalSupportEmail, 'Technical Issue')}
      class="col-span-2 font-mono text-sm text-slate-900 hover:underline dark:text-slate-200 sm:col-span-1"
    >
      {technicalSupportLabel}
    </a>
  </div>

  <div class="grid grid-cols-[2.5rem_1fr] items-center gap-4 p-4 sm:grid-cols-[2.5rem_1fr_auto] sm:gap-6">
    <div class="flex h-10 w-10 items-center justify-center rounded-full bg-slate-200 text-slate-700 dark:bg-slate-700 dark:text-slate-200">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14,2 14,8 20,8"/>
      </svg>
    </div>
    <div>
      <dt class="font-semibold text-slate-900 dark:text-slate-100">Policy Support</dt>
      <dd class="text-sm text-slate-600 dark:text-slate-400">Priority access, queueing, access exceptions</dd>
    </div>
    <a
      href={mailto('compute-governance-p1@aicentre.dk', 'Policy Question')}
      class="col-span-2 font-mono text-sm text-slate-900 hover:underline dark:text-slate-200 sm:col-span-1"
    >
      compute-governance-p1@aicentre.dk
    </a>
  </div>

  <div class="grid grid-cols-[2.5rem_1fr] items-center gap-4 p-4 sm:grid-cols-[2.5rem_1fr_auto] sm:gap-6">
    <div class="flex h-10 w-10 items-center justify-center rounded-full bg-slate-200 text-slate-700 dark:bg-slate-700 dark:text-slate-200">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
      </svg>
    </div>
    <div>
      <dt class="font-semibold text-slate-900 dark:text-slate-100">General Questions</dt>
      <dd class="text-sm text-slate-600 dark:text-slate-400">Open chat with the P1 compute community</dd>
    </div>
    <div class="col-span-2 flex flex-wrap items-center gap-2 text-sm sm:col-span-1 sm:justify-end">
      <a
        href="https://pioneercentreforai.slack.com/channels/compute"
        target="_blank"
        rel="noopener noreferrer"
        class="rounded bg-slate-200 px-2 py-0.5 font-mono text-slate-900 no-underline hover:bg-slate-300 dark:bg-slate-700 dark:text-slate-100 dark:hover:bg-slate-600"
      >
        #compute
      </a>
      <a
        href="https://pioneercentreforai.slack.com/channels/ask-compute"
        target="_blank"
        rel="noopener noreferrer"
        class="rounded bg-slate-200 px-2 py-0.5 font-mono text-slate-900 no-underline hover:bg-slate-300 dark:bg-slate-700 dark:text-slate-100 dark:hover:bg-slate-600"
      >
        #ask-compute
      </a>
    </div>
  </div>

  {#if showComputeCoordinator}
    <div class="grid grid-cols-[2.5rem_1fr] items-center gap-4 p-4 sm:grid-cols-[2.5rem_1fr_auto] sm:gap-6">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-slate-200 text-slate-700 dark:bg-slate-700 dark:text-slate-200">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
          <circle cx="12" cy="7" r="4"/>
        </svg>
      </div>
      <div>
        <dt class="font-semibold text-slate-900 dark:text-slate-100">Compute Coordinator</dt>
        <dd class="text-sm text-slate-600 dark:text-slate-400">
          Onboarding, workload guidance, general questions. Always happy to help
        </dd>
        <!-- Works on every cluster, unlike the DTU-only DOI: acknowledging the
             coordinator credits the person rather than the facility. -->
        <dd class="mt-1 text-xs text-slate-500 dark:text-slate-400">
          Helped with your work? Acknowledge
          <a
            href={COORDINATOR_ORCID}
            target="_blank"
            rel="noopener noreferrer"
            class="font-mono text-slate-600 hover:underline dark:text-slate-300"
          >
            ORCID 0000-0002-2740-1651
          </a>
        </dd>
      </div>
      <div class="col-span-2 flex flex-wrap items-center gap-2 text-sm sm:col-span-1 sm:justify-end">
        <a
          href={SURVEY_URL}
          target="_blank"
          rel="noopener noreferrer"
          class="rounded bg-slate-800 px-2 py-0.5 font-mono text-white no-underline hover:bg-slate-700 dark:bg-slate-200 dark:text-slate-900 dark:hover:bg-slate-300"
        >
          workload survey
        </a>
        <a
          href={mailto(computeCoordinatorEmail, 'General Inquiry')}
          class="font-mono text-slate-900 hover:underline dark:text-slate-200"
        >
          {computeCoordinatorEmail}
        </a>
      </div>
    </div>
  {/if}
</dl>
