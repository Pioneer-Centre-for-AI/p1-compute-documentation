import { existsSync, readFileSync } from 'node:fs';
import { basename } from 'node:path';
import type { Ctx } from '../context';
import { CLUSTER_NAMES, GEFION, type ClusterId } from '../check/clusters';
import { loginShell, shq } from '../exec';
import { remotePath, withProvider, type Reachable } from '../session/login';
import { fail } from '../ui/say';
import { makeStyle } from '../ui/style';
import { saveRecord } from './records';
import { directiveWords, PROVIDER_OF, schedulerOf, sets } from './script';

export type SubmitRequest = { script: string; provider?: ClusterId; dir?: string };

const DTU_QUEUE = 'p1';

export async function submit(ctx: Ctx, req: SubmitRequest): Promise<number> {
  const local = existsSync(req.script);
  const text = local ? readFileSync(req.script, 'utf8') : '';
  const scheduler = local ? schedulerOf(text) : null;

  const written = scheduler ? PROVIDER_OF[scheduler] : undefined;
  if (written === 'ngc')
    return fail(
      ctx,
      `${req.script} is a PBS script, for ${CLUSTER_NAMES.ngc}, which is air-gapped.`,
      ['Submit it with qsub on its remote desktop.']
    );
  if (written && req.provider && req.provider !== written)
    return fail(
      ctx,
      `${req.script} is written for ${CLUSTER_NAMES[written]} (${scheduler === 'lsf' ? '#BSUB' : '#SBATCH'} lines), not ${CLUSTER_NAMES[req.provider]}.`
    );

  return withProvider(ctx, { asked: written ?? req.provider }, async (login, config) => {
    const { provider } = login;
    const dir = remotePath(login, req.dir ?? '');
    let script = remotePath({ home: login.home, base: dir }, req.script);
    let body = text;
    if (!local) {
      const read = await login.master.run(`cat ${shq(script)}`, { timeoutMs: 15_000 });
      if (read.code !== 0)
        return fail(
          ctx,
          `${req.script} is neither on this machine nor at ${script} on ${login.alias}.`
        );
      body = read.stdout;
    } else {
      const stamp = new Date().toISOString().replace(/[-:]/g, '').replace(/\..*$/, '');
      script = `${login.data}/jobs/${stamp}-${basename(req.script)}`;
      const up = await login.master.run(
        loginShell(`mkdir -p ${shq(`${login.data}/jobs`)} && cat > ${shq(script)}`),
        { input: text, timeoutMs: 30_000 }
      );
      if (up.code !== 0)
        return fail(ctx, `cannot copy ${req.script} to ${login.alias}: ${up.stderr.trim()}`);
    }

    const account = config.gefion?.account ?? GEFION.account;
    const words = directiveWords(body, provider === 'dtu' ? 'lsf' : 'slurm');
    const job = queueCommand(provider, words, { dir, logs: `${login.data}/logs`, script, account });
    const r = await login.master.run(loginShell(job.command), { timeoutMs: 60_000 });
    const said = (r.stdout + '\n' + r.stderr).trim();
    const id =
      provider === 'dtu'
        ? /Job <(\d+)> is submitted/.exec(said)?.[1]
        : /^(\d+)(?:;\S+)?$/m.exec(r.stdout.trim())?.[1];
    if (r.code !== 0 || !id)
      return fail(
        ctx,
        `${provider === 'dtu' ? 'bsub' : 'sbatch'} did not take the job:`,
        said.split('\n').filter(Boolean).slice(-6)
      );

    const fill = (path?: string) => path?.replace(/%J|%j/g, id);
    saveRecord(ctx.paths, {
      provider,
      id,
      name: basename(req.script),
      out: fill(job.out),
      err: fill(job.err),
      submittedAt: new Date().toISOString()
    });

    const s = makeStyle(ctx.env, ctx.out);
    const queue = provider === 'dtu' ? /to queue <([^>]+)>/.exec(said)?.[1] : undefined;
    ctx.out.write(
      `${s.symbol('ok')} Submitted job ${s.bold(id)} to ${CLUSTER_NAMES[provider]}${queue ? ` (queue ${queue})` : ''}\n`
    );
    ctx.out.write(
      s.dim(`  aicentre logs ${id} -f follows its output; aicentre list jobs shows where it is.\n`)
    );
    return 0;
  });
}

type Where = { dir: string; logs: string; script: string; account: string };

function queueCommand(
  provider: Reachable,
  words: string[],
  { dir, logs, script, account }: Where
): { command: string; out?: string; err?: string } {
  if (provider === 'dtu') {
    const flags: string[] = [];
    const out = sets(words, ['-o', '-oo']) ? undefined : `${logs}/%J.out`;
    const err = sets(words, ['-e', '-eo']) ? undefined : `${logs}/%J.err`;
    if (!sets(words, ['-q'])) flags.push('-q', DTU_QUEUE);
    if (out) flags.push('-o', out);
    if (err) flags.push('-e', err);
    const command = `mkdir -p ${shq(logs)} && cd ${shq(dir)} && bsub ${flags.map(shq).join(' ')} < ${shq(script)}`;
    return { command, out, err };
  }
  const flags = ['--parsable', `--chdir=${dir}`];
  const out = sets(words, ['-o', '--output']) ? undefined : `${logs}/%j.out`;
  if (!sets(words, ['-A', '--account'])) flags.push(`--account=${account}`);
  if (out) flags.push(`--output=${out}`);
  return {
    command: `mkdir -p ${shq(logs)} && sbatch ${flags.map(shq).join(' ')} ${shq(script)}`,
    out
  };
}
