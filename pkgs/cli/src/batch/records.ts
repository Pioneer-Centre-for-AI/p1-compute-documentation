import { mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import type { Paths } from '../paths';
import type { Reachable } from '../session/login';

export type JobRecord = {
  provider: Reachable;
  id: string;
  name: string;
  out?: string;
  err?: string;
  submittedAt: string;
};

function dir(paths: Paths): string {
  return join(paths.state, 'jobs');
}

const KEEP_MS = 30 * 86_400_000;

export function saveRecord(paths: Paths, r: JobRecord): void {
  mkdirSync(dir(paths), { recursive: true });
  for (const name of readdirSync(dir(paths))) {
    try {
      const old = JSON.parse(readFileSync(join(dir(paths), name), 'utf8')) as JobRecord;
      if (Date.now() - Date.parse(old.submittedAt) > KEEP_MS)
        rmSync(join(dir(paths), name), { force: true });
    } catch {
      rmSync(join(dir(paths), name), { force: true });
    }
  }
  writeFileSync(join(dir(paths), `${r.provider}-${r.id}.json`), JSON.stringify(r, null, 2) + '\n');
}

export function findRecord(paths: Paths, id: string, provider?: string): JobRecord | null {
  let names: string[];
  try {
    names = readdirSync(dir(paths));
  } catch {
    return null;
  }
  const name = names.find(
    (n) => n.endsWith(`-${id}.json`) && (!provider || n.startsWith(`${provider}-`))
  );
  if (!name) return null;
  try {
    return JSON.parse(readFileSync(join(dir(paths), name), 'utf8')) as JobRecord;
  } catch {
    return null;
  }
}

export function dropRecord(paths: Paths, provider: string, id: string): void {
  rmSync(join(dir(paths), `${provider}-${id}.json`), { force: true });
}
