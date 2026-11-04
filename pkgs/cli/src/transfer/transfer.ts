import type { Ctx } from '../context';
import { CLUSTER_NAMES, type ClusterId } from '../check/clusters';
import { attach, loginShell, run, shq, which } from '../exec';
import { sharedSocket } from '../remote/master';
import { notReachable, remotePath, withProvider } from '../session/login';
import { fail } from '../ui/say';
import { makeStyle } from '../ui/style';

export type TransferRequest = {
  direction: 'push' | 'pull';
  from: string;
  to?: string;
  provider?: ClusterId;
};

export function rsyncFlags(versionText: string): string[] {
  const m = /rsync\s+version\s+(\d+)\.(\d+)/.exec(versionText);
  const modern = m ? Number(m[1]) > 3 || (Number(m[1]) === 3 && Number(m[2]) >= 1) : false;
  return modern
    ? ['-az', '--partial', '--protect-args', '--info=progress2']
    : ['-az', '--partial', '--progress'];
}

export async function transfer(ctx: Ctx, req: TransferRequest): Promise<number> {
  if (!which('rsync', ctx.env))
    return fail(ctx, 'rsync is not installed on this machine.', [
      'Install it with your package manager; macOS ships one.'
    ]);
  const why = req.provider && notReachable(req.provider);
  if (why)
    return fail(ctx, `${why} Its staging SFTP endpoint is in the gallery's XFR-DAT example.`);

  return withProvider(ctx, { asked: req.provider }, async (login) => {
    if (login.local)
      return fail(
        ctx,
        `you are on ${CLUSTER_NAMES[login.provider]} already; copy with cp or rsync here.`
      );
    const version = await run('rsync', ['--version'], { env: ctx.env, timeoutMs: 5_000 });
    const rsh = `ssh -S ${shq(sharedSocket(ctx, login.alias))} -o ControlMaster=no`;

    let src: string;
    let dst: string;
    if (req.direction === 'push') {
      const into = `${remotePath(login, req.to ?? '').replace(/\/+$/, '')}/`;
      const made = await login.master.run(loginShell(`mkdir -p ${shq(into)}`), {
        timeoutMs: 15_000
      });
      if (made.code !== 0)
        return fail(ctx, `cannot create ${into} on ${login.alias}: ${made.stderr.trim()}`);
      src = req.from.replace(/\/+$/, '');
      dst = `${login.alias}:${into}`;
    } else {
      src = `${login.alias}:${remotePath(login, req.from)}`;
      dst = req.to ?? '.';
    }

    const s = makeStyle(ctx.env, ctx.out);
    ctx.out.write(`${s.dim(`${src} → ${dst}`)}\n`);
    const code = await attach(
      'rsync',
      [...rsyncFlags(version.stdout), '-e', rsh, src, dst],
      ctx.env
    );
    ctx.out.write(
      code === 0
        ? `${s.symbol('ok')} Copied to ${dst}\n`
        : `${s.symbol('fail')} rsync stopped (exit ${code}); run it again to pick up where it left off.\n`
    );
    return code === 0 ? 0 : 1;
  });
}
