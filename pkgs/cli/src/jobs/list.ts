import { existsSync } from 'node:fs';
import { loadConfigOrSay } from '../config';
import type { Ctx } from '../context';
import {
  CLUSTER_NAMES,
  DTU,
  GEFION,
  SHORT_NAMES,
  tshLogin,
  type ClusterId
} from '../check/clusters';
import { classifySshFailure, providerContext } from '../check/probes/connect';
import { all, expandPath, get } from '../check/probes/ssh';
import { certValidity, duration } from '../check/probes/tsh';
import { resolver, type Resolver } from '../check/aliases';
import { inUse } from '../check/select';
import { loginShell, run, type RunResult } from '../exec';
import { LocalShell } from '../remote/shell';
import { isOpen, KEEP, logIn, sharedSocket } from '../remote/master';
import { here } from '../session/here';
import { liveNotes } from '../session/state';
import { LiveArea } from '../ui/live';
import { makeStyle, type Style } from '../ui/style';
import { table } from '../ui/table';
import { bjobsCommand, noJobs, parseBjobs } from './lsf';
import { parseSqueue, squeueCommand } from './slurm';
import { byState, type Job } from './types';

type Queryable = 'dtu' | 'gefion';

export type ClusterJobs = {
  id: Queryable;
  name: string;
  ok: boolean;
  error?: string;
  hint?: string[];
  jobs: Job[];
  login?: string;
};

export type ListOptions = {
  clusters: ClusterId[];
  json: boolean;
};

const NGC_NOTE = 'air-gapped, so its queue is only visible on its remote desktop (qstat)';

async function query(ctx: Ctx, id: Queryable, hosts: Resolver): Promise<ClusterJobs> {
  const base = { id, name: CLUSTER_NAMES[id], jobs: [] as Job[] };
  const command = loginShell(id === 'dtu' ? bjobsCommand() : squeueCommand());
  if (here(ctx) === id)
    return parse(id, base, await new LocalShell(ctx).run(command, { timeoutMs: 30_000 }));

  const alias = await hosts.alias(id);
  if (!alias) {
    const login = id === 'dtu' ? DTU.login : GEFION.login;
    const hint = ['aicentre check shows the block to add.'];
    return { ...base, ok: false, error: `no Host in ~/.ssh/config points at ${login}`, hint };
  }

  const tsh = id === 'gefion' ? await teleport(ctx, hosts, alias) : { user: '' };
  if (tsh.left !== undefined && tsh.left <= 0) {
    const error = `your Teleport login expired ${duration(tsh.left)} ago`;
    return { ...base, ok: false, error, hint: [tshLogin(tsh.user)] };
  }

  const r = await overSsh(ctx, alias, command);
  if ((r.code === 255 || r.timedOut) && !(id === 'dtu' && noJobs(r.stderr))) {
    const context = providerContext(id, alias, { user: tsh.user, timedOut: r.timedOut });
    const f = classifySshFailure(r.stderr, context);
    const typed = f.kind === 'password' || f.kind === 'host-key-unknown';
    return {
      ...base,
      ok: false,
      error: `cannot reach ${alias}: ${f.message}`,
      hint: typed ? f.hint : [...f.hint, 'aicentre check --connect looks closer.'],
      ...(typed ? { login: alias } : {})
    };
  }
  return parse(id, base, r);
}

async function teleport(
  ctx: Ctx,
  hosts: Resolver,
  alias: string
): Promise<{ user: string; left?: number }> {
  const res = await hosts.sshG(alias);
  if (!res.ok) return { user: '' };
  const user = get(res.cfg, 'user') ?? '';
  const where = { home: ctx.paths.home, localUser: ctx.username, remoteUser: user };
  const cert = all(res.cfg, 'certificatefile').map((p) => expandPath(p, where))[0];
  if (!cert || !existsSync(cert)) return { user };
  const v = await certValidity(cert, ctx.env);
  return { user, left: v?.to ? v.to.getTime() - Date.now() : Infinity };
}

async function overSsh(ctx: Ctx, alias: string, command: string): Promise<RunResult> {
  const socket = sharedSocket(ctx, alias);
  const shared = (await isOpen(ctx, alias, socket)) ? ['-S', socket, '-o', 'ControlMaster=no'] : [];
  const args = ['-o', 'BatchMode=yes', '-o', 'ConnectTimeout=10', ...shared, '-T', alias, command];
  return run('ssh', args, { env: ctx.env, timeoutMs: 30_000, detached: true });
}

function parse(id: Queryable, base: Omit<ClusterJobs, 'ok'>, r: RunResult): ClusterJobs {
  if (id === 'dtu' && noJobs(r.stdout + r.stderr)) return { ...base, ok: true };
  if (r.code !== 0) {
    const last = r.stderr.trim().split('\n').filter(Boolean).at(-1) ?? `exit ${r.code}`;
    return { ...base, ok: false, error: `${id === 'dtu' ? 'bjobs' : 'squeue'} failed: ${last}` };
  }
  const jobs = (id === 'dtu' ? parseBjobs(r.stdout) : parseSqueue(r.stdout)).sort(byState);
  return { ...base, ok: true, jobs };
}

export type Row = Job & { provider: Queryable };

export function merge(results: ClusterJobs[]): Row[] {
  return results
    .flatMap((r) => r.jobs.map((j) => ({ ...j, provider: r.id })))
    .sort((a, b) => byState(a, b) || a.provider.localeCompare(b.provider));
}

const HEAD = ['PROVIDER', 'JOB', 'STATE', 'QUEUE', 'NAME', 'NODE', 'ELAPSED', 'LEFT', ''];

function jobTable(rows: Row[], s: Style, held: Set<string>): string[] {
  const cells = rows.map((j) => [
    SHORT_NAMES[j.provider],
    j.id,
    j.state === 'other' ? j.rawState.toLowerCase() : j.state,
    j.queue,
    j.name.length > 28 ? `${j.name.slice(0, 27)}…` : j.name,
    j.node || '-',
    j.elapsed || '-',
    j.left || '-',
    j.provider === 'dtu' && held.has(j.id) ? 'this laptop' : j.reason
  ]);
  const paint = { running: s.green, pending: s.yellow } as Record<string, (t: string) => string>;
  return table(cells, {
    head: HEAD,
    headStyle: s.dim,
    style: (text, r, c) => {
      if (c === 2) return (paint[rows[r]!.state] ?? s.dim)(text);
      return c === HEAD.length - 1 ? s.dim(text) : text;
    }
  });
}

export function render(
  results: ClusterJobs[],
  s: Style,
  held: Set<string>,
  ngc: boolean
): string[] {
  const rows = merge(results);
  const answered = results.filter((r) => r.ok);
  const lines: string[] = [];
  if (rows.length) {
    const count = rows.length === 1 ? '1 job' : `${rows.length} jobs`;
    lines.push(`${s.bold('Your jobs on P1 compute')}  ${s.dim(count)}`, ...jobTable(rows, s, held));
  } else if (answered.length) {
    const asked = answered.map((r) => SHORT_NAMES[r.id]).join(' and ');
    lines.push(`${s.bold('No jobs on P1 compute')}  ${s.dim(`asked ${asked}`)}`);
  }
  const failed = results.filter((r) => !r.ok);
  if (failed.length) lines.push('');
  for (const c of failed) {
    lines.push(
      `${s.symbol('fail')} ${SHORT_NAMES[c.id]}: ${c.error}`,
      ...(c.hint ?? []).map((h) => `    ${s.dim(h)}`)
    );
  }
  if (ngc) lines.push('', `${s.symbol('info')} ${s.dim(`${CLUSTER_NAMES.ngc} is ${NGC_NOTE}.`)}`);
  return lines;
}

export async function listJobs(ctx: Ctx, opts: ListOptions): Promise<number> {
  const config = loadConfigOrSay(ctx);
  if (!config) return 2;
  const s = makeStyle(ctx.env, ctx.out);
  const hosts = resolver(ctx, config);

  const explicit = opts.clusters.length > 0;
  const wanted = (id: Queryable) =>
    explicit ? opts.clusters.includes(id) : inUse(id, ctx, config, hosts);
  const candidates: Queryable[] = ['dtu', 'gefion'];
  const picked = await Promise.all(candidates.map(wanted));
  const chosen = candidates.filter((_, i) => picked[i]);
  const ngc = explicit ? opts.clusters.includes('ngc') : Boolean(config.ngc);

  if (chosen.length === 0 && !ngc) {
    if (opts.json) ctx.out.write(JSON.stringify({ jobs: [], providers: [] }, null, 2) + '\n');
    else
      ctx.out.write(
        'No P1 provider is set up on this machine yet. aicentre check --all shows what each one needs.\n'
      );
    return 0;
  }

  const live =
    !opts.json && chosen.length && ctx.out.isTTY && ctx.env.TERM !== 'dumb'
      ? new LiveArea(ctx.out, (f) => [
          `${s.spinner(f)} Asking ${chosen.map((id) => SHORT_NAMES[id]).join(' and ')} for your jobs`
        ])
      : undefined;
  live?.start();
  const results = await Promise.all(chosen.map((id) => query(ctx, id, hosts)));
  live?.stop();

  if (ctx.interactive && !opts.json) {
    for (const [i, r] of results.entries()) {
      if (!r.login) continue;
      ctx.out.write(
        `${r.login} asks for more than your key. Log in here; the login stays open for ${KEEP.replace('m', ' min')}.\n`
      );
      const done = await logIn(ctx, r.login);
      if (done.ok) results[i] = await query(ctx, r.id, hosts);
    }
  }

  if (opts.json) {
    const doc = {
      jobs: merge(results),
      providers: results.map(({ jobs: _jobs, ...status }) => status)
    };
    ctx.out.write(JSON.stringify(doc, null, 2) + '\n');
  } else {
    const held = new Set(liveNotes(ctx.paths).keys());
    ctx.out.write(
      render(results, s, held, ngc)
        .map((l) => l + '\n')
        .join('')
    );
  }
  return results.every((r) => r.ok) ? 0 : 1;
}
