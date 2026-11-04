import { shq } from '../exec';
import { clock, type Job } from './types';

const FORMAT = '%i|%T|%P|%j|%N|%M|%L|%r';

export function squeueCommand(): string {
  return `squeue --noheader -u "$USER" --format=${shq(FORMAT)}`;
}

const STATES: Record<string, Job['state']> = {
  RUNNING: 'running',
  COMPLETING: 'running',
  CONFIGURING: 'running',
  PENDING: 'pending',
  SUSPENDED: 'suspended'
};

function duration(text: string): string {
  const m = /^(?:(\d+)-)?(?:(\d+):)?(\d+):(\d{2})$/.exec(text.trim());
  if (!m) return '';
  const seconds =
    Number(m[1] ?? 0) * 86_400 + Number(m[2] ?? 0) * 3_600 + Number(m[3]) * 60 + Number(m[4]);
  return seconds ? clock(seconds) : '';
}

function firstNode(list: string): string {
  const t = list.trim();
  if (!t || t === '(null)') return '';
  const m = /^([^,[]+)(?:\[(\d+))?/.exec(t);
  return m ? `${m[1]}${m[2] ?? ''}` : t;
}

export function parseSqueue(text: string): Job[] {
  const jobs: Job[] = [];
  for (const line of text.split('\n')) {
    if (!/^\d+(?:_\d+|_\[[^\]]*\])?\|/.test(line.trim())) continue;
    const [
      id = '',
      state = '',
      partition = '',
      name = '',
      nodes = '',
      used = '',
      left = '',
      ...reason
    ] = line.trim().split('|');
    const kind = STATES[state] ?? 'other';
    jobs.push({
      id,
      state: kind,
      rawState: state,
      queue: partition,
      name,
      node: firstNode(nodes),
      reason:
        kind === 'pending'
          ? reason
              .join('|')
              .replace(/^None$/, '')
              .trim()
          : '',
      elapsed: duration(used),
      left: duration(left)
    });
  }
  return jobs;
}
