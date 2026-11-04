import { loadConfigOrSay } from '../config';
import type { Ctx } from '../context';
import { killAll } from '../exec';
import { JsonReporter, LiveReporter, PlainReporter, summarize, type Reporter } from '../ui/report';
import { makeStyle } from '../ui/style';
import type { ClusterId } from './clusters';
import { runChecks } from './run';

export async function check(
  ctx: Ctx,
  opts: { clusters: ClusterId[]; connect: boolean; json: boolean }
): Promise<number> {
  const config = loadConfigOrSay(ctx);
  if (!config) return 2;

  const style = makeStyle(ctx.env, ctx.out);
  const live = !opts.json && Boolean(ctx.out.isTTY) && ctx.env.TERM !== 'dumb';
  const reporter: Reporter = opts.json
    ? new JsonReporter(ctx.out, { version: ctx.version, connect: opts.connect })
    : live
      ? new LiveReporter(style, ctx.out)
      : new PlainReporter(style, ctx.out);

  const onInt = () => {
    reporter.abort();
    killAll();
    process.exit(130);
  };
  process.on('SIGINT', onInt);
  try {
    const reports = await runChecks(
      ctx,
      config,
      { clusters: opts.clusters, connect: opts.connect },
      reporter
    );
    const summary = summarize(reports, style, opts.connect);
    reporter.finish(reports, summary);
    return summary.ok ? 0 : 1;
  } catch (error) {
    reporter.abort();
    throw error;
  } finally {
    process.off('SIGINT', onInt);
  }
}
