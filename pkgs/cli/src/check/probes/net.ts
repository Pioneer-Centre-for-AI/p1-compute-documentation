import { lookup } from 'node:dns/promises';
import { Socket } from 'node:net';

export type NetResult =
  | { ok: true }
  | {
      ok: false;
      reason: 'dns' | 'timeout' | 'refused' | 'unreachable' | 'closed' | 'other';
      message: string;
    };

export type TcpProbe = typeof tcpProbe;

export function reachTimeout(connect: boolean): number {
  return connect ? 5_000 : 3_000;
}

export async function tcpProbe(
  host: string,
  port: number,
  opts: { timeoutMs?: number } = {}
): Promise<NetResult> {
  const timeoutMs = opts.timeoutMs ?? 5_000;
  let address: string;
  try {
    address = (await withTimeout(lookup(host), timeoutMs)).address;
  } catch (error) {
    if ((error as Error).message === 'timeout')
      return {
        ok: false,
        reason: 'timeout',
        message: `looking up ${host} took over ${timeoutMs / 1000} s`
      };
    return { ok: false, reason: 'dns', message: `cannot look up ${host}` };
  }

  return new Promise((resolve) => {
    const socket = new Socket();
    let done = false;
    const finish = (result: NetResult) => {
      if (done) return;
      done = true;
      clearTimeout(timer);
      socket.destroy();
      resolve(result);
    };
    const timer = setTimeout(
      () =>
        finish({
          ok: false,
          reason: 'timeout',
          message: `no answer from ${host}:${port} within ${timeoutMs / 1000} s`
        }),
      timeoutMs
    );
    socket.once('error', (error: NodeJS.ErrnoException) => {
      const reason =
        error.code === 'ECONNREFUSED'
          ? 'refused'
          : error.code === 'EHOSTUNREACH' || error.code === 'ENETUNREACH'
            ? 'unreachable'
            : 'other';
      const what =
        reason === 'refused'
          ? 'refused the connection'
          : reason === 'unreachable'
            ? 'is unreachable'
            : `failed (${error.code ?? error.message})`;
      finish({ ok: false, reason, message: `${host}:${port} ${what}` });
    });
    socket.connect(port, address, () => finish({ ok: true }));
  });
}

function withTimeout<T>(promise: Promise<T>, ms: number): Promise<T> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error('timeout')), ms);
    promise.then(
      (v) => (clearTimeout(timer), resolve(v)),
      (e) => (clearTimeout(timer), reject(e))
    );
  });
}
