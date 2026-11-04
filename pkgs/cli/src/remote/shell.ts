import type { Ctx } from '../context';
import { attach, run, stream, type RunResult, type Streamed } from '../exec';

export interface Shell {
  run(command: string, opts?: { input?: string; timeoutMs?: number }): Promise<RunResult>;
  stream(command: string, onLine: (line: string) => void, tty?: boolean): Streamed;
  attach(command: string): Promise<number>;
  forward(local: number, host: string, port: number): Promise<RunResult>;
  close(): Promise<void>;
}

export function unwrapLogin(command: string): string {
  const m = /^bash -lc ('(?:[^']|'\\'')*'|[^\s']+)$/.exec(command);
  return m ? `bash -c ${m[1]}` : command;
}

export class LocalShell implements Shell {
  constructor(private readonly ctx: Ctx) {}

  run(command: string, opts: { input?: string; timeoutMs?: number } = {}): Promise<RunResult> {
    return run('bash', ['-c', unwrapLogin(command)], { env: this.ctx.env, ...opts });
  }

  stream(command: string, onLine: (line: string) => void): Streamed {
    return stream('bash', ['-c', unwrapLogin(command)], this.ctx.env, onLine);
  }

  attach(command: string): Promise<number> {
    return attach('bash', ['-c', unwrapLogin(command)], this.ctx.env);
  }

  async forward(): Promise<RunResult> {
    return {
      code: 1,
      stdout: '',
      stderr: 'there is no port to forward on the cluster itself',
      timedOut: false,
      missing: false
    };
  }

  async close(): Promise<void> {}
}
