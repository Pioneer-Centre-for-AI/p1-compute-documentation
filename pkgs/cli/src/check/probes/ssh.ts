import { run } from '../../exec';
import type { Env } from '../../context';

export type SshVersion = { major: number; minor: number; text: string };

export function parseSshVersion(output: string): SshVersion | null {
  const m = /OpenSSH_(?:for_Windows_)?(\d+)\.(\d+)/.exec(output);
  if (!m) return null;
  return { major: Number(m[1]), minor: Number(m[2]), text: `${m[1]}.${m[2]}` };
}

export function atLeast(v: SshVersion, major: number, minor: number): boolean {
  return v.major > major || (v.major === major && v.minor >= minor);
}

export async function sshVersion(env: Env): Promise<SshVersion | null> {
  const r = await run('ssh', ['-V'], { env, timeoutMs: 5_000 });
  if (r.missing) return null;
  return parseSshVersion(r.stderr + r.stdout);
}

export type SshConfig = Map<string, string[]>;

export function parseSshG(output: string): SshConfig {
  const cfg: SshConfig = new Map();
  for (const line of output.split('\n')) {
    const i = line.indexOf(' ');
    if (i <= 0) continue;
    const key = line.slice(0, i).toLowerCase();
    const value = line.slice(i + 1).trim();
    const values = cfg.get(key);
    if (values) values.push(value);
    else cfg.set(key, [value]);
  }
  return cfg;
}

export function get(cfg: SshConfig, key: string): string | undefined {
  return cfg.get(key)?.[0];
}

export function all(cfg: SshConfig, key: string): string[] {
  return cfg.get(key) ?? [];
}

export function words(cfg: SshConfig, key: string): string[] {
  return all(cfg, key)
    .flatMap((v) => v.split(/\s+/))
    .filter(Boolean);
}

export type SshGResult = { ok: true; cfg: SshConfig } | { ok: false; error: string };

export async function sshG(alias: string, env: Env): Promise<SshGResult> {
  const r = await run('ssh', ['-G', alias], { env, timeoutMs: 5_000 });
  if (r.missing) return { ok: false, error: 'ssh is not installed' };
  if (r.code !== 0) {
    const error =
      r.stderr.trim().split('\n').filter(Boolean).slice(0, 3).join('\n') ||
      `ssh -G exited ${r.code}`;
    return { ok: false, error };
  }
  return { ok: true, cfg: parseSshG(r.stdout) };
}

export function isConfigured(cfg: SshConfig, alias: string): boolean {
  const hostname = get(cfg, 'hostname');
  return hostname !== undefined && hostname !== alias.toLowerCase();
}

export type Tokens = {
  home: string;
  localUser: string;
  remoteUser?: string;
  host?: string;
  uid?: number;
};

export function expandPath(path: string, t: Tokens): string {
  let p = path.startsWith('~/') ? t.home + path.slice(1) : path === '~' ? t.home : path;
  p = p.replace(/%([%dhilru])/g, (match, token: string) => {
    switch (token) {
      case '%':
        return '%';
      case 'd':
        return t.home;
      case 'u':
        return t.localUser;
      case 'r':
        return t.remoteUser ?? match;
      case 'h':
        return t.host ?? match;
      case 'i':
        return t.uid === undefined ? match : String(t.uid);
      default:
        return match;
    }
  });
  return p;
}
