import type { Config } from '../config';
import type { Ctx } from '../context';
import type { Resolver } from '../check/aliases';
import type { ClusterId } from '../check/clusters';
import { inUse } from '../check/select';
import { liveNotes } from '../session/state';
import { notReachable, type Reachable } from '../session/login';
import { findRecord } from './records';

export async function providerOf(
  ctx: Ctx,
  config: Config,
  hosts: Resolver,
  id: string,
  asked?: ClusterId
): Promise<{ provider: Reachable } | { error: string }> {
  if (asked) {
    const why = notReachable(asked);
    return why ? { error: why } : { provider: asked as Reachable };
  }
  const record = findRecord(ctx.paths, id);
  if (record) return { provider: record.provider };
  if (liveNotes(ctx.paths).has(id)) return { provider: 'dtu' };
  const used: Reachable[] = [];
  for (const p of ['dtu', 'gefion'] as const) if (await inUse(p, ctx, config, hosts)) used.push(p);
  if (used.length === 1) return { provider: used[0]! };
  if (used.length === 0)
    return {
      error:
        'no P1 provider is set up on this machine yet; aicentre check --all shows what each one needs.'
    };
  return {
    error: `job ${id} was not submitted from here, so which provider ran it? Add --provider ${used.join(' or --provider ')}.`
  };
}
