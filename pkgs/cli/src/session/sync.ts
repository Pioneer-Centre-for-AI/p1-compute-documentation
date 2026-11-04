import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import type { Config } from '../config';
import type { Ctx } from '../context';
import { loginShell, shq } from '../exec';
import type { Shell } from '../remote/shell';
import type { Style } from '../ui/style';
import { isVersion } from '../install/release';
import { pathLine } from '../install/self-install';
import type { Reachable } from './login';

export const INSTALL_URL = 'https://hpc.aicentre.dk/install';

export const VERSION_PROBE =
  'echo "@@AICENTRE $("$HOME/.local/bin/aicentre" --version 2>/dev/null)"';

export const PATH_PROBE =
  'case ":$PATH:" in *":$HOME/.local/bin:"*) echo "@@BINPATH yes";; esac; echo "@@SHELL ${SHELL##*/}"';

export type Probe = { there: string; binOnPath: boolean; shell: string };

export type SyncAction = 'none' | 'install' | 'newer-there' | 'off';

export function decide(ctx: Ctx, config: Config, provider: Reachable, there: string): SyncAction {
  if (ctx.env.AICENTRE_SYNC === '0' || config[provider]?.sync === false) return 'off';
  if (!isVersion(ctx.version) || ctx.version.includes('-dev')) return 'off';
  if (!isVersion(there)) return 'install';
  const order = Bun.semver.order(there, ctx.version);
  return order < 0 ? 'install' : order > 0 ? 'newer-there' : 'none';
}

export function syncCommand(version: string, provider: Reachable): string {
  const install =
    `curl -fsSL ${INSTALL_URL} | AICENTRE_VERSION=${version} sh >"$0/sync.log" 2>&1 && ` +
    `printf '%s\\n' ${provider} > "\${XDG_DATA_HOME:-$HOME/.local/share}/aicentre/provider"; rmdir "$0/sync.lock"`;
  return [
    's="${XDG_STATE_HOME:-$HOME/.local/state}/aicentre"',
    'mkdir -p "$s"',
    'find "$s" -maxdepth 1 -name sync.lock -mmin +30 -exec rmdir {} \\; 2>/dev/null',
    `if mkdir "$s/sync.lock" 2>/dev/null; then nohup sh -c ${shq(install)} "$s" >/dev/null 2>&1 </dev/null & fi`,
    'true'
  ].join('; ');
}

type Remote = { alias: string; version: string; at: string; pathHinted?: boolean };
type Remotes = Partial<Record<Reachable, Remote>>;

function remotesFile(ctx: Ctx): string {
  return join(ctx.paths.state, 'remotes.json');
}

export function readRemotes(ctx: Ctx): Remotes {
  try {
    return JSON.parse(readFileSync(remotesFile(ctx), 'utf8')) as Remotes;
  } catch {
    return {};
  }
}

function remember(ctx: Ctx, provider: Reachable, patch: Partial<Remote> & { alias: string }): void {
  const remotes = readRemotes(ctx);
  remotes[provider] = {
    version: ctx.version,
    at: new Date().toISOString(),
    ...remotes[provider],
    ...patch
  };
  mkdirSync(ctx.paths.state, { recursive: true });
  writeFileSync(remotesFile(ctx), JSON.stringify(remotes, null, 2) + '\n');
}

export async function sync(
  ctx: Ctx,
  config: Config,
  shell: Shell,
  provider: Reachable,
  alias: string,
  probe: Probe,
  style: Style
): Promise<void> {
  const { there } = probe;
  const action = decide(ctx, config, provider, there);
  if (action === 'off') return;
  if (action === 'newer-there') {
    ctx.out.write(
      style.dim(
        `  ${alias} has aicentre ${there}, newer than this one (${ctx.version}); aicentre upgrade updates it here.\n`
      )
    );
  }
  if (action === 'install') {
    const r = await shell.run(loginShell(syncCommand(ctx.version, provider)), {
      timeoutMs: 15_000
    });
    if (r.code !== 0) return;
    remember(ctx, provider, { alias, version: ctx.version, at: new Date().toISOString() });
    ctx.out.write(
      style.dim(
        `  ${there ? `Updating aicentre ${there} → ${ctx.version}` : `Installing aicentre ${ctx.version}`} on ${alias} in the background, so it is there too.\n`
      )
    );
  }

  const present = action === 'install' || Boolean(there);
  if (present && !probe.binOnPath && !readRemotes(ctx)[provider]?.pathHinted) {
    ctx.out.write(
      style.dim(
        `  ~/.local/bin is not on your PATH on ${alias}, so call it as ~/.local/bin/aicentre there, or run this once on ${alias}:\n`
      )
    );
    ctx.out.write(`      ${style.cyan(pathLine(probe.shell))}\n`);
    remember(ctx, provider, { alias, pathHinted: true });
  }
}
