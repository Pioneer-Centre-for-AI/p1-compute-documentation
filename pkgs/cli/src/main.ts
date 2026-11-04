import { renderHelp } from './cli/help';
import { parseCommand, UsageError, type Command } from './cli/args';
import { check } from './check/command';
import { processCtx, type Ctx } from './context';
import { session } from './session/command';
import { submit } from './batch/submit';
import { logs } from './batch/logs';
import { cancel } from './batch/cancel';
import { chat } from './chat/command';
import { transfer } from './transfer/transfer';
import { gallery } from './gallery/gallery';
import { listJobs } from './jobs/list';
import { selfInstall } from './install/self-install';
import { uninstall } from './install/uninstall';
import { upgrade } from './install/upgrade';
import { makeStyle } from './ui/style';

export async function main(argv: string[], ctx: Ctx): Promise<number> {
  let command: Command;
  try {
    command = parseCommand(argv);
  } catch (error) {
    if (!(error instanceof UsageError)) throw error;
    ctx.err.write(`aicentre: ${error.message}\nRun aicentre help for usage.\n`);
    return 2;
  }

  switch (command.name) {
    case 'help':
      ctx.out.write(renderHelp(command.topic, makeStyle(ctx.env, ctx.out), ctx.version));
      return 0;
    case 'version':
      ctx.out.write(`${ctx.version}\n`);
      return 0;
    case 'check':
      return check(ctx, command);
    case 'upgrade':
      return upgrade(ctx, { version: command.version });
    case 'uninstall':
      return uninstall(ctx, { purge: command.purge });
    case 'session':
      return session(ctx, command.session);
    case 'submit':
      return submit(ctx, command);
    case 'logs':
      return logs(ctx, command);
    case 'cancel':
      return cancel(ctx, command);
    case 'chat':
      return chat(ctx, command.args);
    case 'transfer':
      return transfer(ctx, command);
    case 'gallery':
      return gallery(ctx, command.gallery);
    case 'list':
      return listJobs(ctx, { clusters: command.clusters, json: command.json });
    case 'self-install':
      return selfInstall(ctx);
  }
}

if (import.meta.main) {
  main(process.argv.slice(2), processCtx()).then(
    (code) => process.exit(code),
    (error: Error) => {
      process.stderr.write(`aicentre: ${error.message}\n`);
      process.exit(1);
    }
  );
}
