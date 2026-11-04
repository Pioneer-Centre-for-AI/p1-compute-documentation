import { shq } from '../exec';
import { clock, type Job } from './types';

const FIELDS = 'jobid stat queue job_name exec_host run_time time_left pend_reason';

export function bjobsCommand(): string {
  return `bjobs -noheader -o ${shq(`${FIELDS} delimiter='|'`)}`;
}

const STATES: Record<string, Job['state']> = {
  RUN: 'running',
  PEND: 'pending',
  PSUSP: 'suspended',
  USUSP: 'suspended',
  SSUSP: 'suspended'
};

function firstHost(execHost: string): string {
  const first = execHost.split(':')[0] ?? '';
  return first === '-' ? '' : first.replace(/^\d+\*/, '');
}

function runTime(text: string): string {
  const m = /^(\d+) second/.exec(text.trim());
  return m && m[1] !== '0' ? clock(Number(m[1])) : '';
}

function timeLeft(text: string): string {
  const m = /^(\d+):(\d{2})/.exec(text.trim());
  return m ? clock(Number(m[1]) * 3_600 + Number(m[2]) * 60) : '';
}

export function parseBjobs(text: string): Job[] {
  const jobs: Job[] = [];
  for (const line of text.split('\n')) {
    if (!/^\d+\|/.test(line.trim())) continue;
    const [id = '', stat = '', queue = '', name = '', host = '', run = '', left = '', ...reason] =
      line.trim().split('|');
    jobs.push({
      id,
      state: STATES[stat] ?? 'other',
      rawState: stat,
      queue,
      name,
      node: firstHost(host),
      reason: stat === 'PEND' ? reason.join('|').replace(/^-$/, '').trim() : '',
      elapsed: runTime(run),
      left: timeLeft(left)
    });
  }
  return jobs;
}

export function noJobs(text: string): boolean {
  return /No unfinished job found|No matching job found|No job found/i.test(text);
}
