export type Job = {
  id: string;
  state: 'running' | 'pending' | 'suspended' | 'other';
  rawState: string;
  queue: string;
  name: string;
  node: string;
  reason: string;
  elapsed: string;
  left: string;
};

const ORDER: Record<Job['state'], number> = { running: 0, suspended: 1, pending: 2, other: 3 };

export function byState(a: Job, b: Job): number {
  return ORDER[a.state] - ORDER[b.state] || Number(a.id) - Number(b.id);
}

export function clock(seconds: number): string {
  const d = Math.floor(seconds / 86_400);
  const h = Math.floor((seconds % 86_400) / 3_600);
  const m = Math.floor((seconds % 3_600) / 60);
  const s = Math.floor(seconds % 60);
  const mmss = `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
  return d ? `${d}-${String(h).padStart(2, '0')}:${mmss}` : `${h}:${mmss}`;
}
