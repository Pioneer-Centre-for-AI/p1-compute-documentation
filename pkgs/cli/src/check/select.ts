import type { Config } from '../config';
import type { Ctx } from '../context';
import { GEFION, type ClusterId } from './clusters';
import type { Resolver } from './aliases';
import { hasProfile, tshHome } from './probes/tsh';
import { here } from '../session/here';

export async function inUse(
  id: ClusterId,
  ctx: Ctx,
  config: Config,
  r: Resolver
): Promise<boolean> {
  if (config[id] || here(ctx) === id) return true;
  if (id === 'gefion' && hasProfile(tshHome(ctx.env, ctx.paths.home), GEFION.proxy)) return true;
  return (await r.alias(id)) !== null;
}
