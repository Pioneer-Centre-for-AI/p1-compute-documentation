import type { Ctx } from '../context';
import { resolver } from '../check/aliases';
import type { ClusterId } from '../check/clusters';
import { loginShell, shq } from '../exec';
import { loadConfigOrSay } from '../config';
import { withLogin, type Login } from '../session/login';
import { fail } from '../ui/say';
import { findRecord } from './records';
import { providerOf } from './which';

export type LogsRequest = {
  id: string;
  follow: boolean;
  lines: number;
  err: boolean;
  provider?: ClusterId;
};

export function scontrolField(text: string, field: 'StdOut' | 'StdErr'): string | undefined {
  return new RegExp(`\\b${field}=(\\S+)`).exec(text)?.[1];
}

async function askScheduler(login: Login, id: string, err: boolean): Promise<string | undefined> {
  if (login.provider === 'dtu') {
    const r = await login.master.run(
      loginShell(`bjobs -noheader -o ${shq(`${err ? 'error_file' : 'output_file'}`)} ${shq(id)}`),
      { timeoutMs: 20_000 }
    );
    const path = r.stdout.trim().split('\n').at(-1)?.trim();
    if (!path || path === '-' || /not found/i.test(path)) return undefined;
    const absolute = path.startsWith('/') ? path : `${login.base}/${path}`;
    return absolute.replace(/%J/g, id);
  }
  const r = await login.master.run(loginShell(`scontrol show job ${shq(id)}`), {
    timeoutMs: 20_000
  });
  return scontrolField(r.stdout, err ? 'StdErr' : 'StdOut');
}

export async function logs(ctx: Ctx, req: LogsRequest): Promise<number> {
  const config = loadConfigOrSay(ctx);
  if (!config) return 2;
  const hosts = resolver(ctx, config);
  const where = await providerOf(ctx, config, hosts, req.id, req.provider);
  if ('error' in where) return fail(ctx, where.error);
  const record = findRecord(ctx.paths, req.id, where.provider);

  return withLogin(ctx, config, hosts, where.provider, async (login) => {
    const file =
      (req.err ? record?.err : record?.out) ?? (await askScheduler(login, req.id, req.err));
    if (!file)
      return fail(
        ctx,
        `${login.provider === 'dtu' ? 'LSF' : 'Slurm'} does not say where job ${req.id} writes its ${req.err ? 'errors' : 'output'}.`,
        ['A job that finished a while ago is no longer known to it.']
      );
    if (req.follow) {
      ctx.out.write(`Following ${file}. Ctrl-C stops following; the job keeps running.\n`);
      return login.master.attach(`tail -n ${req.lines} -F ${shq(file)}`);
    }
    const present = await login.master.run(`test -f ${shq(file)}`, { timeoutMs: 15_000 });
    if (present.code !== 0) {
      ctx.out.write(
        `Job ${req.id} has not written to ${file} yet. It may still be waiting; aicentre list jobs shows its state.\n`
      );
      return 0;
    }
    const r = await login.master.run(`tail -n ${req.lines} ${shq(file)}`, { timeoutMs: 30_000 });
    ctx.out.write(r.stdout);
    return r.code === 0 ? 0 : 1;
  });
}
