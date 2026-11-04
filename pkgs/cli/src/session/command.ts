import type { Ctx } from '../context';
import { freePort, openUrl } from '../browser';
import { interactive, type InteractiveRequest } from './interactive';

export async function session(ctx: Ctx, req: InteractiveRequest): Promise<number> {
  const controller = new AbortController();
  const onSignal = () => {
    if (req.app === 'shell') return;
    if (controller.signal.aborted) process.exit(130);
    controller.abort();
  };
  const signals = ['SIGINT', 'SIGTERM', 'SIGHUP'] as const;
  for (const sig of signals) process.on(sig, onSignal);
  try {
    return await interactive(ctx, req, {
      signal: controller.signal,
      openUrl: (url) => openUrl(ctx, url),
      freePort
    });
  } finally {
    for (const sig of signals) process.off(sig, onSignal);
  }
}
