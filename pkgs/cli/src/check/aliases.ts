import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { basename, dirname, isAbsolute, join } from 'node:path';
import type { Config } from '../config';
import type { Ctx } from '../context';
import { DTU, GEFION, type ClusterId } from './clusters';
import { get, isConfigured, sshG as sshGRun, type SshGResult } from './probes/ssh';

export const TARGETS: Partial<Record<ClusterId, { host: string; port?: number }>> = {
  dtu: { host: DTU.login },
  gefion: { host: GEFION.login, port: GEFION.port }
};

const DEFAULTS: Partial<Record<ClusterId, string>> = { dtu: DTU.alias, gefion: GEFION.alias };

function expandInclude(pattern: string, home: string): string[] {
  let p = pattern.startsWith('~/') ? join(home, pattern.slice(2)) : pattern;
  if (!isAbsolute(p)) p = join(home, '.ssh', p);
  const name = basename(p);
  if (!/[*?[]/.test(name)) return existsSync(p) ? [p] : [];
  const re = new RegExp(
    '^' +
      name
        .replace(/[.+^${}()|\\]/g, '\\$&')
        .replace(/\*/g, '.*')
        .replace(/\?/g, '.') +
      '$'
  );
  try {
    return readdirSync(dirname(p))
      .filter((f) => re.test(f))
      .sort()
      .map((f) => join(dirname(p), f))
      .filter((f) => statSync(f).isFile());
  } catch {
    return [];
  }
}

function words(line: string): string[] {
  const out: string[] = [];
  for (const m of line.matchAll(/"([^"]*)"|([^\s=]+)/g)) out.push(m[1] ?? m[2]!);
  return out;
}

export function aliasesFor(
  host: string,
  home: string,
  file = join(home, '.ssh', 'config')
): string[] {
  const found: string[] = [];
  const seen = new Set<string>();
  const want = host.toLowerCase();

  const scan = (path: string, depth: number) => {
    if (depth > 16 || seen.has(path)) return;
    seen.add(path);
    let text: string;
    try {
      text = readFileSync(path, 'utf8');
    } catch {
      return;
    }
    let names: string[] = [];
    for (const raw of text.split('\n')) {
      const line = raw.replace(/(^|\s)#.*$/, '').trim();
      if (!line) continue;
      const [keyword = '', ...args] = words(line);
      switch (keyword.toLowerCase()) {
        case 'host':
          names = args.filter((a) => !/[*?!]/.test(a));
          break;
        case 'match':
          names = [];
          break;
        case 'hostname':
          if (args[0]?.toLowerCase() === want)
            for (const n of names) if (!found.includes(n)) found.push(n);
          break;
        case 'include':
          for (const pattern of args)
            for (const f of expandInclude(pattern, home)) scan(f, depth + 1);
          break;
      }
    }
  };
  scan(file, 0);
  return found;
}

function pointsAt(res: SshGResult, target: { host: string; port?: number }): boolean {
  if (!res.ok || get(res.cfg, 'hostname')?.toLowerCase() !== target.host) return false;
  return target.port === undefined || Number(get(res.cfg, 'port') ?? 22) === target.port;
}

export async function findAlias(
  id: ClusterId,
  ctx: Ctx,
  config: Config,
  sshG: (alias: string) => Promise<SshGResult>
): Promise<string | null> {
  const configured = config[id]?.alias;
  if (configured) return configured;
  const target = TARGETS[id];
  if (!target) return null;
  const fallback = DEFAULTS[id];
  if (fallback && pointsAt(await sshG(fallback), target)) return fallback;

  const candidates = aliasesFor(target.host, ctx.paths.home);
  let hostOnly: string | null = null;
  for (const alias of candidates) {
    const res = await sshG(alias);
    if (pointsAt(res, target)) return alias;
    if (!hostOnly && res.ok && get(res.cfg, 'hostname')?.toLowerCase() === target.host)
      hostOnly = alias;
  }
  if (hostOnly) return hostOnly;
  if (fallback) {
    const res = await sshG(fallback);
    if (res.ok && isConfigured(res.cfg, fallback)) return fallback;
  }
  return null;
}

export type Resolver = {
  sshG(alias: string): Promise<SshGResult>;
  alias(id: ClusterId): Promise<string | null>;
};

export function resolver(ctx: Ctx, config: Config): Resolver {
  const configs = new Map<string, Promise<SshGResult>>();
  const aliases = new Map<ClusterId, Promise<string | null>>();
  const r: Resolver = {
    sshG(alias) {
      let p = configs.get(alias);
      if (!p) configs.set(alias, (p = sshGRun(alias, ctx.env)));
      return p;
    },
    alias(id) {
      let p = aliases.get(id);
      if (!p) aliases.set(id, (p = findAlias(id, ctx, config, r.sshG)));
      return p;
    }
  };
  return r;
}
