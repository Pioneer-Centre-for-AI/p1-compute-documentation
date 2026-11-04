import { spawn } from 'node:child_process';
import { createServer } from 'node:net';
import type { Ctx } from './context';
import { which } from './exec';

export async function openUrl(ctx: Ctx, url: string): Promise<void> {
  const linuxDesktop = Boolean(ctx.env.DISPLAY || ctx.env.WAYLAND_DISPLAY);
  const cmd = ctx.platform.os === 'darwin' ? 'open' : linuxDesktop ? 'xdg-open' : null;
  if (!cmd || !which(cmd, ctx.env)) return;
  try {
    spawn(cmd, [url], {
      env: ctx.env as NodeJS.ProcessEnv,
      stdio: 'ignore',
      detached: true
    }).unref();
  } catch {}
}

export function freePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => {
      const address = server.address();
      const port = typeof address === 'object' && address ? address.port : 0;
      server.close(() => resolve(port));
    });
  });
}

export function link(url: string, tty: boolean): string {
  return tty ? `\x1b]8;;${url}\x1b\\${url}\x1b]8;;\x1b\\` : url;
}
