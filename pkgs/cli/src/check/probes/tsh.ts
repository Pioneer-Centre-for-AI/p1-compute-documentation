import { existsSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { run } from '../../exec';
import type { Env } from '../../context';

export function tshHome(env: Env, home: string): string {
  return env.TELEPORT_HOME || join(home, '.tsh');
}

export function hasProfile(tsh: string, proxy: string): boolean {
  return existsSync(join(tsh, `${proxy}.yaml`));
}

export function profileUsers(tsh: string, proxy: string): string[] {
  try {
    return readdirSync(join(tsh, 'keys', proxy))
      .filter((name) => name.endsWith('-ssh'))
      .map((name) => name.slice(0, -'-ssh'.length));
  } catch {
    return [];
  }
}

export type TshVersion = { major: number; minor: number; text: string };

export async function tshVersion(env: Env): Promise<TshVersion | 'missing' | 'unknown'> {
  const r = await run('tsh', ['version', '--client', '--format=json'], { env, timeoutMs: 5_000 });
  if (r.missing) return 'missing';
  try {
    const text = String((JSON.parse(r.stdout) as { version?: unknown }).version ?? '');
    const m = /^v?(\d+)\.(\d+)/.exec(text);
    if (m) return { major: Number(m[1]), minor: Number(m[2]), text };
  } catch {}
  return 'unknown';
}

export type Validity = { from: Date | null; to: Date | null };

export function parseValidity(output: string): Validity | null {
  const m = /Valid:\s+(?:from\s+(\S+)\s+to\s+(\S+)|(forever))/.exec(output);
  if (!m) return null;
  if (m[3]) return { from: null, to: null };
  const date = (s: string | undefined) => (s && s !== 'forever' ? new Date(s) : null);
  return { from: date(m[1]), to: date(m[2]) };
}

export async function certValidity(cert: string, env: Env): Promise<Validity | null> {
  const r = await run('ssh-keygen', ['-L', '-f', cert], { env, timeoutMs: 5_000 });
  if (r.code !== 0) return null;
  return parseValidity(r.stdout);
}

export function duration(ms: number): string {
  const min = Math.round(Math.abs(ms) / 60_000);
  if (min < 60) return `${min} min`;
  const h = Math.floor(min / 60);
  if (h < 48) return min % 60 ? `${h} h ${min % 60} min` : `${h} h`;
  return `${Math.round(h / 24)} days`;
}
