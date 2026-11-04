import type { Ctx } from '../context';
import { CLUSTER_NAMES, type ClusterId } from '../check/clusters';
import { link } from '../browser';
import { shq } from '../exec';
import { LiveArea } from '../ui/live';
import { fail } from '../ui/say';
import { Steps } from '../ui/steps';
import type { Style } from '../ui/style';
import { notReachable, remotePath, withProvider, type Login } from './login';
import {
  bsubCommand,
  launcherArgs,
  loginShell,
  parseLine,
  QUEUE,
  type App,
  type Server
} from './lsf';
import { dropNote, saveNote } from './state';

export type InteractiveRequest = {
  app: App;
  path: string;
  gpu: boolean;
  provider?: ClusterId;
  server: Server;
  minutes: number;
  cpus: number;
  memPerCpu: string;
  browser: boolean;
};

export type SessionDeps = {
  signal: AbortSignal;
  openUrl(url: string): Promise<void>;
  freePort(): Promise<number>;
};

const NAMES = { shell: 'Shell', marimo: 'marimo', code: 'VS Code' } as const;

function aborted(signal: AbortSignal): Promise<'abort'> {
  return new Promise((resolve) => {
    if (signal.aborted) resolve('abort');
    else signal.addEventListener('abort', () => resolve('abort'), { once: true });
  });
}

function noGpuOn(asked?: ClusterId): string | null {
  if (!asked || asked === 'dtu') return null;
  if (asked === 'gefion')
    return `${CLUSTER_NAMES.gefion} runs GPU work as batch jobs; aicentre submit sends one.`;
  return notReachable(asked);
}

function sessionCommand(login: Login, r: InteractiveRequest, bsubMode: '-I' | '-Is'): string {
  if (r.gpu) return loginShell(bsubCommand(r, login.launcher, login.data, login.base, bsubMode));
  const args = launcherArgs(r, login.launcher, login.data, login.base, '127.0.0.1');
  return loginShell(args.map(shq).join(' '));
}

export async function interactive(
  ctx: Ctx,
  r: InteractiveRequest,
  deps: SessionDeps
): Promise<number> {
  const why = r.gpu ? noGpuOn(r.provider) : null;
  if (why) return fail(ctx, why);

  return withProvider(ctx, { asked: r.gpu ? 'dtu' : r.provider }, async (login) => {
    if (login.local && !(r.app === 'shell' && r.gpu))
      return fail(
        ctx,
        r.app === 'shell'
          ? `you are already on the ${CLUSTER_NAMES[login.provider]} login node. aicentre shell --gpu opens one on a GPU node.`
          : `${NAMES[r.app]} opens in your browser, so run this from your laptop.`
      );
    const req = { ...r, path: remotePath(login, r.path) };
    if (r.app !== 'shell') return serve(ctx, login, req, deps);
    if (r.gpu)
      ctx.out.write(
        `Asking ${QUEUE} for a GPU for ${r.minutes} min. The prompt appears on the GPU node; exit ends the job.\n`
      );
    return login.master.attach(sessionCommand(login, req, '-Is'));
  });
}

type Heard = {
  jobid?: string;
  pid?: string;
  error?: string;
  logs: string[];
  recent: string[];
  got: Record<string, string>;
};

function listen(
  ctx: Ctx,
  r: InteractiveRequest,
  steps: Steps,
  heard: Heard,
  onReady: () => void
): (line: string) => void {
  return (line) => {
    const e = parseLine(line);
    if (e.kind === 'submitted') {
      heard.jobid = e.jobid;
      const startedAt = new Date().toISOString();
      saveNote(ctx.paths, { jobid: e.jobid, app: r.app, pid: process.pid, startedAt });
      steps.step(`Waiting for a GPU on ${QUEUE} (job ${e.jobid})`);
    } else if (e.kind === 'started') {
      steps.step(`Running on ${e.host}`);
    } else if (e.kind === 'text') {
      if (e.line.trim()) heard.recent.push(e.line.trim());
      if (heard.recent.length > 8) heard.recent.shift();
    } else if (e.kind === 'tag') {
      if (e.name === 'PID') heard.pid = e.value;
      else if (e.name === 'GPU') steps.note(`GPU: ${e.value}`);
      else if (e.name === 'STATUS') steps.step(e.value.charAt(0).toUpperCase() + e.value.slice(1));
      else if (e.name === 'ERROR') heard.error = e.value;
      else if (e.name === 'LOG') heard.logs.push(e.value);
      else if (e.name === 'HOST' || e.name === 'PORT' || e.name === 'PATH') {
        heard.got[e.name] = e.value;
        if (heard.got.HOST && heard.got.PORT && heard.got.PATH) onReady();
      }
    }
  };
}

async function serve(
  ctx: Ctx,
  login: Login,
  r: InteractiveRequest,
  deps: SessionDeps
): Promise<number> {
  const steps = new Steps(ctx.env, ctx.out);
  const name = NAMES[r.app];
  const heard: Heard = { logs: [], recent: [], got: {} };
  let markReady: () => void = () => {};
  const ready = new Promise<'ready'>((resolve) => (markReady = () => resolve('ready')));

  steps.step(r.gpu ? `Submitting to ${QUEUE}` : `Starting ${name} on ${login.alias}`);
  const job = login.master.stream(
    sessionCommand(login, r, '-I'),
    listen(ctx, r, steps, heard, markReady),
    !r.gpu
  );
  const exited = job.exit.then(() => 'exit' as const);

  const end = async (code: number, message?: string) => {
    if (heard.jobid) {
      await login.master.run(loginShell(`bkill ${heard.jobid}`), { timeoutMs: 15_000 });
      dropNote(ctx.paths, heard.jobid);
    } else if (heard.pid) {
      await login.master.run(`kill ${shq(heard.pid)}`, { timeoutMs: 15_000 });
    }
    job.child.kill('SIGTERM');
    if (message) ctx.out.write(`${message}\n`);
    return code;
  };
  const stopped = () => {
    if (heard.jobid) return `Ended job ${heard.jobid}.`;
    return r.gpu ? 'Stopped before the job was submitted.' : `Stopped ${name} on ${login.alias}.`;
  };

  const first = await Promise.race([ready, exited, aborted(deps.signal)]);
  if (first === 'abort') {
    steps.done();
    return end(130, stopped());
  }
  if (first === 'exit') {
    steps.fail(
      `${name} did not start: ${whyNotStarted(r, heard)}`,
      [...heard.logs, ...(heard.error ? [] : heard.recent)].slice(-8)
    );
    steps.done();
    if (heard.jobid) dropNote(ctx.paths, heard.jobid);
    return 1;
  }

  const local = await deps.freePort();
  const target = r.gpu ? heard.got.HOST! : '127.0.0.1';
  const fwd = await login.master.forward(local, target, Number(heard.got.PORT));
  if (fwd.code !== 0) {
    steps.fail(`cannot forward a port to ${target}:${heard.got.PORT}: ${fwd.stderr.trim()}`);
    steps.done();
    return end(1);
  }
  const url = `http://127.0.0.1:${local}${heard.got.PATH}`;
  if (heard.jobid) {
    const startedAt = new Date().toISOString();
    saveNote(ctx.paths, { jobid: heard.jobid, app: r.app, pid: process.pid, startedAt, url });
  }
  steps.done();

  const until = Date.now() + r.minutes * 60_000;
  ctx.out.write(readyBlock(ctx, steps.style, login, r, heard, url));
  if (r.browser) await deps.openUrl(url);

  const clock = runningClock(ctx, steps.style, r.gpu ? until : undefined);
  clock?.start();
  const last = await Promise.race([exited, aborted(deps.signal)]);
  clock?.stop();
  if (last === 'abort') return end(130, stopped());
  if (heard.jobid) dropNote(ctx.paths, heard.jobid);
  const reason =
    r.gpu && Date.now() >= until - 60_000
      ? 'its time ran out'
      : (heard.error ?? 'the server stopped');
  ctx.out.write(
    `${steps.style.symbol('info')} ${heard.jobid ? `Job ${heard.jobid}` : name} ended: ${reason}.\n`
  );
  return 0;
}

function whyNotStarted(r: InteractiveRequest, heard: Heard): string {
  if (heard.error) return heard.error;
  if (!r.gpu) return 'it stopped before it was ready';
  return heard.jobid ? 'the job ended before the server was ready' : 'bsub did not submit the job';
}

function readyBlock(
  ctx: Ctx,
  s: Style,
  login: Login,
  r: InteractiveRequest,
  heard: Heard,
  url: string
): string {
  const where = r.gpu
    ? heard.got.HOST
    : `${login.alias}, the ${CLUSTER_NAMES[login.provider]} login node`;
  const jobid = heard.jobid;
  const hint = r.gpu
    ? `  Job ${jobid ?? '?'} has ${r.minutes} min. Ctrl-C here ends it, and so does aicentre cancel ${jobid ?? '<job>'}.`
    : '  It runs until you press Ctrl-C. For real compute, add --gpu or submit a batch job.';
  const lines = [
    '',
    `${s.symbol('ok')} ${s.bold(`${NAMES[r.app]} is ready`)} on ${where}`,
    '',
    `    ${link(url, Boolean(ctx.out.isTTY))}`,
    '',
    s.dim(hint),
    ''
  ];
  return lines.join('\n') + '\n';
}

function runningClock(ctx: Ctx, s: Style, until?: number): LiveArea | undefined {
  if (!ctx.out.isTTY || ctx.env.TERM === 'dumb') return undefined;
  const left = () =>
    until === undefined
      ? 'Ctrl-C ends it'
      : `${Math.max(0, Math.ceil((until - Date.now()) / 60_000))} min left`;
  return new LiveArea(ctx.out, () => [s.dim(`  ${s.green('●')} running · ${left()}`)], 1_000);
}
