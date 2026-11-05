import { basename } from 'node:path';
import type { Ctx } from '../context';
import { tilde } from '../paths';
import { fail } from '../ui/say';
import { makeStyle } from '../ui/style';
import {
  activate,
  linkBin,
  placeBinary,
  pruneVersions,
  removeIfEmpty,
  safeRemove,
  smokeTest,
  versionDir,
  writeManifest,
  type LinkResult
} from './layout';

export function commit(ctx: Ctx, version: string, created: string[] = []): LinkResult['result'] {
  activate(ctx.paths, version);
  const link = linkBin(ctx.paths);
  writeManifest(ctx.paths, version, ctx.platform, [...created, ...link.created]);
  pruneVersions(ctx.paths, version);
  return link.result;
}

export type PathHint = { add: string; now: string | null };

export function pathHint(ctx: Ctx): PathHint | null {
  const dirs = (ctx.env.PATH ?? '').split(':');
  if (dirs.includes(ctx.paths.binDir)) return null;
  const shell = basename(ctx.env.SHELL ?? '');
  const dir = tilde(ctx.paths.binDir, ctx.paths.home).replace(/^~/, '$HOME');
  return { add: pathLine(shell, dir), now: sessionLine(shell, dir) };
}

export function pathLine(shell: string, dir = '$HOME/.local/bin'): string {
  if (shell === 'fish') return `fish_add_path ${dir.replace('$HOME', '~')}`;
  if (shell === 'tcsh' || shell === 'csh') return `echo 'set path = (${dir} $path)' >> ~/.cshrc`;
  const rc = shell === 'zsh' ? '~/.zshrc' : shell === 'bash' ? '~/.bashrc' : '~/.profile';
  return `echo 'export PATH="${dir}:$PATH"' >> ${rc}`;
}

export function sessionLine(shell: string, dir = '$HOME/.local/bin'): string | null {
  if (shell === 'fish') return null;
  if (shell === 'tcsh' || shell === 'csh') return `set path = (${dir} $path)`;
  return `export PATH="${dir}:$PATH"`;
}

export async function selfInstall(ctx: Ctx): Promise<number> {
  const s = makeStyle(ctx.env, ctx.out);
  if (!ctx.compiled) {
    ctx.err.write('aicentre: self-install runs from a release binary; install.sh calls it.\n');
    return 2;
  }
  const home = ctx.paths.home;
  const { path: binary, created } = placeBinary(ctx.paths, ctx.version, ctx.execPath);
  const problem = await smokeTest(binary, ctx.version, ctx.env);
  if (problem) {
    safeRemove(versionDir(ctx.paths, ctx.version), ctx.paths);
    removeIfEmpty(created);
    return fail(ctx, `the new binary does not run: ${problem}`);
  }
  const link = commit(ctx, ctx.version, created);

  ctx.out.write(
    `${s.symbol('ok')} Installed aicentre ${ctx.version} in ${tilde(ctx.paths.data, home)}\n`
  );
  if (link === 'foreign') {
    ctx.out.write(
      `${s.symbol('warn')} ${tilde(ctx.paths.binLink, home)} already exists and is not ours, so it was left alone.\n`
    );
    ctx.out.write(
      `  Run aicentre as ${tilde(binary.replace(/versions\/[^/]+/, 'current'), home)}.\n`
    );
  } else {
    ctx.out.write(`  The command is ${tilde(ctx.paths.binLink, home)}.\n`);
    const hint = pathHint(ctx);
    if (hint) {
      const where = hint.now ? ' for new terminals' : '';
      ctx.out.write(
        `${s.symbol('warn')} ${tilde(ctx.paths.binDir, home)} is not on your PATH. Add it${where}:\n`
      );
      ctx.out.write(`    ${s.cyan(hint.add)}\n`);
      if (hint.now) ctx.out.write(`  and for this one:\n    ${s.cyan(hint.now)}\n`);
    }
  }
  ctx.out.write(
    `\nNext, see whether this machine is ready for the clusters:\n    ${s.cyan('aicentre check')}\n`
  );
  return 0;
}
