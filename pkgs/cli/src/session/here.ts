import { existsSync, readFileSync } from 'node:fs';
import { hostname } from 'node:os';
import { join } from 'node:path';
import type { Ctx } from '../context';
import { DTU } from '../check/clusters';
import { which } from '../exec';
import type { Reachable } from './login';

export function providerFile(dataDir: string): string {
  return join(dataDir, 'provider');
}

export function here(ctx: Ctx): Reachable | null {
  const said = ctx.env.AICENTRE_HERE;
  if (said === 'dtu' || said === 'gefion') return said;
  if (said === 'none') return null;
  try {
    const marked = readFileSync(providerFile(ctx.paths.data), 'utf8').trim();
    if (marked === 'dtu' || marked === 'gefion') return marked;
  } catch {}
  if (which('bsub', ctx.env) && existsSync(DTU.projectDir)) return 'dtu';
  const host = hostname().toLowerCase();
  if (which('sbatch', ctx.env) && (host.includes('dcai') || host.startsWith('slurm-login')))
    return 'gefion';
  return null;
}
