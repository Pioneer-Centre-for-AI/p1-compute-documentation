import { spawn } from 'node:child_process';
import { readFileSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import type { Ctx } from '../context';
import { attach, run, stream, type RunResult, type Streamed } from '../exec';
import { ensureRuntimeDir, resolvePaths } from '../paths';

export const KEEP = '30m';

const SOCKET_MAX = 100;

export function sharedSocket(ctx: Ctx, alias: string): string {
  const name = `cm-${new Bun.CryptoHasher('sha1').update(alias).digest('hex').slice(0, 12)}`;
  const path = join(ensureRuntimeDir(ctx.paths, ctx.uid), name);
  if (path.length <= SOCKET_MAX) return path;
  const short = resolvePaths({ ...ctx.env, XDG_RUNTIME_DIR: undefined, TMPDIR: '/tmp' }, ctx.uid);
  return join(ensureRuntimeDir(short, ctx.uid), name);
}

export async function isOpen(
  ctx: Ctx,
  alias: string,
  socket = sharedSocket(ctx, alias)
): Promise<boolean> {
  const r = await run('ssh', ['-S', socket, '-O', 'check', alias], {
    env: ctx.env,
    timeoutMs: 5_000
  });
  return r.code === 0;
}

export async function logIn(
  ctx: Ctx,
  alias: string,
  socket = sharedSocket(ctx, alias)
): Promise<{ ok: true } | { ok: false; stderr: string }> {
  const log = `${socket}.log`;
  rmSync(log, { force: true });
  const batch = ctx.interactive ? [] : ['-o', 'BatchMode=yes'];
  const code = await new Promise<number>((resolve) => {
    const child = spawn(
      'ssh',
      [
        ...batch,
        '-o',
        'ControlMaster=yes',
        '-o',
        `ControlPath=${socket}`,
        '-o',
        `ControlPersist=${KEEP}`,
        '-o',
        'ServerAliveInterval=30',
        '-o',
        'ServerAliveCountMax=4',
        '-E',
        log,
        alias,
        'true'
      ],
      {
        env: ctx.env as NodeJS.ProcessEnv,
        stdio: [ctx.interactive ? 'inherit' : 'ignore', 'ignore', 'ignore']
      }
    );
    child.on('error', () => resolve(127));
    child.on('close', (c) => resolve(c ?? 1));
  });
  let stderr = '';
  try {
    stderr = readFileSync(log, 'utf8');
  } catch {}
  rmSync(log, { force: true });
  if (code === 0 && (await isOpen(ctx, alias, socket))) return { ok: true };
  return { ok: false, stderr: stderr || `ssh exited ${code}` };
}

export class Master {
  private readonly forwards: string[] = [];

  constructor(
    readonly alias: string,
    private readonly socket: string,
    private readonly ctx: Ctx
  ) {}

  private client(...args: string[]): string[] {
    return ['-S', this.socket, '-o', 'ControlMaster=no', ...args];
  }

  async start(): Promise<{ ok: true } | { ok: false; stderr: string }> {
    if (await isOpen(this.ctx, this.alias, this.socket)) return { ok: true };
    return logIn(this.ctx, this.alias, this.socket);
  }

  run(command: string, opts: { input?: string; timeoutMs?: number } = {}): Promise<RunResult> {
    return run('ssh', this.client('-T', this.alias, command), { env: this.ctx.env, ...opts });
  }

  stream(command: string, onLine: (line: string) => void, tty = false): Streamed {
    return stream(
      'ssh',
      this.client(tty ? '-tt' : '-T', '-o', 'LogLevel=ERROR', this.alias, command),
      this.ctx.env,
      onLine
    );
  }

  attach(command: string): Promise<number> {
    return attach(
      'ssh',
      this.client('-t', '-o', 'LogLevel=ERROR', this.alias, command),
      this.ctx.env
    );
  }

  async forward(local: number, host: string, port: number): Promise<RunResult> {
    const spec = `127.0.0.1:${local}:${host}:${port}`;
    const r = await run('ssh', ['-S', this.socket, '-O', 'forward', '-L', spec, this.alias], {
      env: this.ctx.env,
      timeoutMs: 10_000
    });
    if (r.code === 0) this.forwards.push(spec);
    return r;
  }

  async close(): Promise<void> {
    for (const spec of this.forwards.splice(0)) {
      await run('ssh', ['-S', this.socket, '-O', 'cancel', '-L', spec, this.alias], {
        env: this.ctx.env,
        timeoutMs: 5_000
      });
    }
  }
}
