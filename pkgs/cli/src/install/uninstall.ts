import { existsSync, lstatSync } from 'node:fs';
import type { Ctx } from '../context';
import { tilde } from '../paths';
import { isNixStore } from '../platform';
import { confirm } from '../ui/prompt';
import { makeStyle } from '../ui/style';
import { isOurLink, readManifest, removeIfEmpty, removeLink, safeRemove } from './layout';
import { readRemotes } from '../session/sync';
import { CLUSTER_NAMES } from '../check/clusters';

function present(path: string): boolean {
  try {
    lstatSync(path);
    return true;
  } catch {
    return false;
  }
}

export type UninstallOptions = { purge: boolean; ask?: (question: string) => Promise<boolean> };

export async function uninstall(ctx: Ctx, opts: UninstallOptions): Promise<number> {
  const s = makeStyle(ctx.env, ctx.out);
  if (isNixStore(ctx.execPath)) {
    ctx.err.write(
      'aicentre: this aicentre comes from nix; remove it from your Nix configuration instead.\n'
    );
    return 1;
  }
  const home = ctx.paths.home;
  const removed: string[] = [];
  const kept: string[] = [];
  const createdDirs = readManifest(ctx.paths)?.createdDirs ?? [];
  const remotes = Object.entries(readRemotes(ctx));

  if (present(ctx.paths.binLink)) {
    if (isOurLink(ctx.paths.binLink, ctx.paths)) {
      removeLink(ctx.paths.binLink);
      removed.push(ctx.paths.binLink);
    } else {
      kept.push(`${tilde(ctx.paths.binLink, home)} (not ours)`);
    }
  }

  for (const dir of [ctx.paths.data, ctx.paths.state, ctx.paths.cache, ctx.paths.runtime]) {
    if (!existsSync(dir)) continue;
    safeRemove(dir, ctx.paths);
    removed.push(dir);
  }

  if (existsSync(ctx.paths.config)) {
    const question = `Also remove your settings in ${tilde(ctx.paths.config, home)}?`;
    const ask = opts.ask ?? (ctx.interactive ? confirm : undefined);
    const remove = opts.purge || (ask ? await ask(question) : false);
    if (remove) {
      safeRemove(ctx.paths.config, ctx.paths);
      removed.push(ctx.paths.config);
    } else {
      kept.push(tilde(ctx.paths.config, home));
    }
  }

  removeIfEmpty(createdDirs);

  if (removed.length === 0) {
    ctx.out.write(`${s.symbol('ok')} aicentre is not installed here; nothing to remove.\n`);
  } else {
    ctx.out.write(`${s.symbol('ok')} Removed aicentre:\n`);
    for (const path of removed) ctx.out.write(`    ${tilde(path, home)}\n`);
  }
  for (const path of kept) ctx.out.write(`${s.symbol('info')} Kept ${path}\n`);
  if (kept.some((k) => k === tilde(ctx.paths.config, home))) {
    ctx.out.write(s.dim(`  Remove it later with: rm -r ${tilde(ctx.paths.config, home)}\n`));
  }
  for (const [provider, remote] of remotes) {
    ctx.out.write(
      `${s.symbol('info')} aicentre is also installed on ${CLUSTER_NAMES[provider as keyof typeof CLUSTER_NAMES]}. To remove it there:\n`
    );
    ctx.out.write(`    ${s.cyan(`ssh -t ${remote!.alias} '~/.local/bin/aicentre uninstall'`)}\n`);
  }
  return 0;
}
