import { spawn, type ChildProcess } from 'node:child_process';
import type { Env } from './context';

export type RunResult = {
  code: number;
  stdout: string;
  stderr: string;
  timedOut: boolean;
  missing: boolean;
};

export type RunOptions = {
  env: Env;
  timeoutMs?: number;
  detached?: boolean;
  input?: string;
};

const live = new Set<ChildProcess>();

function kill(child: ChildProcess, group: boolean): void {
  if (child.pid === undefined || child.exitCode !== null) return;
  try {
    if (group) process.kill(-child.pid, 'SIGKILL');
    else child.kill('SIGKILL');
  } catch {
    if (group) child.kill('SIGKILL');
  }
}

export function killAll(): void {
  for (const child of live) kill(child, true);
}

export function run(cmd: string, args: string[], opts: RunOptions): Promise<RunResult> {
  return new Promise((resolve) => {
    const detached = opts.detached ?? false;
    const child = spawn(cmd, args, {
      env: opts.env as NodeJS.ProcessEnv,
      stdio: [opts.input === undefined ? 'ignore' : 'pipe', 'pipe', 'pipe'],
      detached
    });
    live.add(child);
    if (opts.input !== undefined) {
      child.stdin?.on('error', () => {});
      child.stdin?.end(opts.input);
    }
    let stdout = '';
    let stderr = '';
    let timedOut = false;
    let settled = false;
    child.stdout?.setEncoding('utf8').on('data', (chunk: string) => (stdout += chunk));
    child.stderr?.setEncoding('utf8').on('data', (chunk: string) => (stderr += chunk));

    const timer = setTimeout(() => {
      timedOut = true;
      kill(child, detached);
    }, opts.timeoutMs ?? 15_000);

    const finish = (result: RunResult) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      live.delete(child);
      resolve(result);
    };

    child.on('error', (error: NodeJS.ErrnoException) => {
      finish({
        code: 127,
        stdout,
        stderr: stderr || error.message,
        timedOut,
        missing: error.code === 'ENOENT'
      });
    });
    child.on('close', (code, signal) => {
      finish({ code: code ?? (signal ? 128 : 1), stdout, stderr, timedOut, missing: false });
    });
  });
}

export function which(cmd: string, env: Env): string | null {
  return Bun.which(cmd, { PATH: env.PATH ?? '' });
}

export type Streamed = { child: ChildProcess; exit: Promise<number> };

export function stream(
  cmd: string,
  args: string[],
  env: Env,
  onLine: (line: string) => void
): Streamed {
  const child = spawn(cmd, args, {
    env: env as NodeJS.ProcessEnv,
    stdio: ['ignore', 'pipe', 'pipe']
  });
  live.add(child);
  const split = () => {
    let rest = '';
    return (chunk: string) => {
      rest += chunk;
      const lines = rest.split(/\r?\n/);
      rest = lines.pop() ?? '';
      for (const line of lines) onLine(line);
    };
  };
  child.stdout?.setEncoding('utf8').on('data', split());
  child.stderr?.setEncoding('utf8').on('data', split());
  const exit = new Promise<number>((resolve) => {
    child.on('error', () => resolve(127));
    child.on('close', (code, signal) => {
      live.delete(child);
      resolve(code ?? (signal ? 128 : 1));
    });
  });
  return { child, exit };
}

export function attach(cmd: string, args: string[], env: Env): Promise<number> {
  return new Promise((resolve) => {
    const child = spawn(cmd, args, { env: env as NodeJS.ProcessEnv, stdio: 'inherit' });
    child.on('error', () => resolve(127));
    child.on('close', (code, signal) => resolve(code ?? (signal ? 128 : 1)));
  });
}

export function shq(word: string): string {
  return /^[A-Za-z0-9_@%+=:,./-]+$/.test(word) ? word : `'${word.replace(/'/g, `'\\''`)}'`;
}

export function loginShell(command: string): string {
  return `bash -lc ${shq(command)}`;
}
