import { shq } from '../exec';

export const QUEUE = 'p1i';
export const JOB_PREFIX = 'aicentre-';

export type App = 'shell' | 'marimo' | 'code';
export type Server = 'vscode' | 'openvscode';

export type Request = {
  app: App;
  path: string;
  server: Server;
  minutes: number;
  cpus: number;
  memPerCpu: string;
};

export function parseMinutes(text: string): number | null {
  const t = text.trim().toLowerCase();
  let m: RegExpExecArray | null;
  if ((m = /^(\d+)m?$/.exec(t))) return Number(m[1]) || null;
  if ((m = /^(\d+):(\d{2})$/.exec(t))) return Number(m[1]) * 60 + Number(m[2]) || null;
  if ((m = /^(?:(\d+)h)?(?:(\d+)m)?$/.exec(t)) && (m[1] || m[2]))
    return Number(m[1] ?? 0) * 60 + Number(m[2] ?? 0) || null;
  return null;
}

export function isMemory(text: string): boolean {
  return /^\d+(?:\.\d+)?[KMGT]B$/i.test(text);
}

export function hm(minutes: number): string {
  return `${Math.floor(minutes / 60)}:${String(minutes % 60).padStart(2, '0')}`;
}

export function bsubCommand(
  r: Request,
  launcher: string,
  data: string,
  base: string,
  interactiveFlag: '-I' | '-Is'
): string {
  const args = [
    interactiveFlag,
    '-q',
    QUEUE,
    '-app',
    'h100app',
    '-J',
    `${JOB_PREFIX}${r.app}`,
    '-env',
    'none',
    '-W',
    hm(r.minutes),
    '-n',
    String(r.cpus),
    '-R',
    `rusage[mem=${r.memPerCpu}]`,
    '-R',
    'span[hosts=1]',
    '-gpu',
    'num=1:mode=shared',
    ...launcherArgs(r, launcher, data, base, '0.0.0.0')
  ];
  return `ESUB_BYPASS=1 ESUB_QUIET=1 bsub ${args.map(shq).join(' ')}`;
}

export function launcherArgs(
  r: Pick<Request, 'app' | 'path' | 'server'>,
  launcher: string,
  data: string,
  base: string,
  bind: string
): string[] {
  return ['bash', '-l', launcher, r.app, r.path, r.server, data, base, bind];
}

export { loginShell } from '../exec';

export type Event =
  | { kind: 'submitted'; jobid: string }
  | { kind: 'queued' }
  | { kind: 'busy' }
  | { kind: 'started'; host: string }
  | { kind: 'tag'; name: string; value: string }
  | { kind: 'text'; line: string };

export function parseLine(line: string): Event {
  let m: RegExpExecArray | null;
  if ((m = /^Job <(\d+)> is submitted/.exec(line))) return { kind: 'submitted', jobid: m[1]! };
  if (line.startsWith('<<Waiting for dispatch')) return { kind: 'queued' };
  if (line.startsWith('LSF is processing')) return { kind: 'busy' };
  if ((m = /^<<Starting on (.+?)>>/.exec(line))) return { kind: 'started', host: m[1]! };
  if ((m = /^@@([A-Z]+) ?(.*)$/.exec(line))) return { kind: 'tag', name: m[1]!, value: m[2] ?? '' };
  return { kind: 'text', line };
}
