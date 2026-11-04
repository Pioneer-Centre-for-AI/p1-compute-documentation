import type { Config } from '../config';
import type { Ctx } from '../context';
import { CLUSTER_IDS, CLUSTER_NAMES, type ClusterId } from './clusters';
import type { CheckEnv } from './env';
import { checkDtu } from './dtu';
import { checkGefion } from './gefion';
import { checkLocal } from './local';
import { checkNgc } from './ngc';
import { resolver } from './aliases';
import { inUse } from './select';
import { here } from '../session/here';
import { Group, type GroupReport, type Sink } from './types';

export type CheckOptions = { clusters: ClusterId[]; connect: boolean };

const CHECKS: Record<ClusterId, (g: Group, c: CheckEnv) => Promise<void>> = {
  dtu: checkDtu,
  gefion: checkGefion,
  ngc: checkNgc
};

function notSetUp(id: ClusterId): string {
  return id === 'ngc'
    ? 'not in config.toml; aicentre check --all checks it too'
    : `no Host in ~/.ssh/config points at it; aicentre check --all shows what to add`;
}

export async function runChecks(
  ctx: Ctx,
  config: Config,
  opts: CheckOptions,
  sink: Sink
): Promise<GroupReport[]> {
  const local = new Group('local', 'This machine', sink);
  const machine = await checkLocal(local, ctx);
  const reports: GroupReport[] = [local.end()];

  const r = resolver(ctx, config);
  const base: CheckEnv = {
    ctx,
    config,
    connect: opts.connect,
    explicit: opts.clusters.length > 0,
    machine,
    sshG: r.sshG,
    alias: r.alias
  };

  const selected = await Promise.all(
    CLUSTER_IDS.map(async (id) =>
      base.explicit
        ? opts.clusters.includes(id)
        : machine.sshUsable && (await inUse(id, ctx, config, r))
    )
  );

  const clusters = await Promise.all(
    CLUSTER_IDS.map(async (id, i) => {
      if (!selected[i] && base.explicit) return null;
      const g = new Group(id, CLUSTER_NAMES[id], sink);
      if (!selected[i]) {
        g.skipped = true;
        g.note = notSetUp(id);
        return g.end();
      }
      if (here(ctx) === id) {
        await g.step('here', 'This machine', () => ({
          status: 'info',
          detail: `is the ${CLUSTER_NAMES[id]} login node; aicentre runs its scheduler directly`
        }));
        return g.end();
      }
      await CHECKS[id](g, base);
      return g.end();
    })
  );
  for (const r of clusters) if (r) reports.push(r);
  return reports;
}
