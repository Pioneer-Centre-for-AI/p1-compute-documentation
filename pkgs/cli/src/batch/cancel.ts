import type { Ctx } from '../context';
import { resolver } from '../check/aliases';
import { CLUSTER_NAMES, type ClusterId } from '../check/clusters';
import { loginShell, shq } from '../exec';
import { loadConfigOrSay } from '../config';
import { withLogin, type Login, type Reachable } from '../session/login';
import { dropNote } from '../session/state';
import { fail } from '../ui/say';
import { makeStyle } from '../ui/style';
import { dropRecord } from './records';
import { providerOf } from './which';

export async function cancel(
  ctx: Ctx,
  req: { ids: string[]; provider?: ClusterId }
): Promise<number> {
  const config = loadConfigOrSay(ctx);
  if (!config) return 2;
  const hosts = resolver(ctx, config);

  const byProvider = new Map<Reachable, string[]>();
  for (const id of req.ids) {
    const where = await providerOf(ctx, config, hosts, id, req.provider);
    if ('error' in where) return fail(ctx, where.error);
    byProvider.set(where.provider, [...(byProvider.get(where.provider) ?? []), id]);
  }

  let code = 0;
  for (const [provider, ids] of byProvider) {
    const done = await withLogin(ctx, config, hosts, provider, (login) => end(ctx, login, ids));
    if (done !== 0) code = 1;
  }
  return code;
}

async function end(ctx: Ctx, login: Login, ids: string[]): Promise<number> {
  const { provider } = login;
  const tool = provider === 'dtu' ? 'bkill' : 'scancel';
  const r = await login.master.run(loginShell(`${tool} ${ids.map(shq).join(' ')}`), {
    timeoutMs: 30_000
  });
  const said = (r.stdout + r.stderr).trim();
  if (r.code !== 0)
    return fail(ctx, `${tool} on ${CLUSTER_NAMES[provider]} said:`, said.split('\n'));

  for (const id of ids) {
    dropRecord(ctx.paths, provider, id);
    if (provider === 'dtu') dropNote(ctx.paths, id);
  }
  const s = makeStyle(ctx.env, ctx.out);
  const jobs = ids.length === 1 ? `job ${ids[0]}` : `jobs ${ids.join(', ')}`;
  ctx.out.write(`${s.symbol('ok')} Cancelled ${jobs} on ${CLUSTER_NAMES[provider]}\n`);
  if (said && provider === 'dtu')
    ctx.out.write(
      s.dim(
        said
          .split('\n')
          .map((l) => `  ${l}`)
          .join('\n') + '\n'
      )
    );
  return 0;
}
