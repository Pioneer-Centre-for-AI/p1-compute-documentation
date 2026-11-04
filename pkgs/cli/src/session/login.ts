import { existsSync, mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { loadConfigOrSay, type Config } from '../config';
import type { Ctx } from '../context';
import { resolver, type Resolver } from '../check/aliases';
import { CLUSTER_NAMES, DTU, GEFION, type ClusterId } from '../check/clusters';
import { classifySshFailure, parseTags, providerContext } from '../check/probes/connect';
import { inUse } from '../check/select';
import { loginShell } from '../exec';
import { isOpen, KEEP, Master, sharedSocket } from '../remote/master';
import { LocalShell, type Shell } from '../remote/shell';
import { fail } from '../ui/say';
import { makeStyle } from '../ui/style';
import { here } from './here';
import { PATH_PROBE, sync, VERSION_PROBE } from './sync';
import launcher from './launcher.sh' with { type: 'text' };

export type Reachable = 'dtu' | 'gefion';

export type Login = {
  provider: Reachable;
  master: Shell;
  local: boolean;
  alias: string;
  user: string;
  home: string;
  base: string;
  data: string;
  launcher: string;
};

const LOGIN: Record<Reachable, string> = { dtu: DTU.login, gefion: GEFION.login };
const NO_PROJECT_DIR = 'Ask the compute coordinator to set up your P1 project directory.';
const NOT_SET_UP = 'no P1 provider is set up on this machine yet.';
const CHECK_ALL = 'aicentre check --all shows what each one needs.';

export function notReachable(id: ClusterId): string | null {
  return id === 'ngc'
    ? `${CLUSTER_NAMES.ngc} is air-gapped: you work on it through its remote desktop.`
    : null;
}

export type Choice = {
  asked?: ClusterId;
  among?: readonly ClusterId[];
  outside?: string;
  none?: [string, ...string[]];
};

export async function pickProvider(
  ctx: Ctx,
  config: Config,
  hosts: Resolver,
  choice: Choice = {}
): Promise<Reachable | { error: string; hints: string[] }> {
  const among = choice.among ?? ['dtu', 'gefion'];
  if (choice.asked) {
    const why = notReachable(choice.asked);
    if (why) return { error: why, hints: [] };
    if (!among.includes(choice.asked)) return { error: choice.outside ?? NOT_SET_UP, hints: [] };
    return choice.asked as Reachable;
  }
  for (const id of among) {
    if (id !== 'ngc' && (await inUse(id, ctx, config, hosts))) return id;
  }
  const [error, ...hints] = choice.none ?? [NOT_SET_UP, CHECK_ALL];
  return { error, hints };
}

export async function withLogin(
  ctx: Ctx,
  config: Config,
  hosts: Resolver,
  provider: Reachable,
  body: (login: Login) => Promise<number>
): Promise<number> {
  const login = await openLogin(ctx, config, provider, hosts);
  if (!login) return 1;
  try {
    return await body(login);
  } finally {
    await login.master.close();
  }
}

export async function withProvider(
  ctx: Ctx,
  choice: Choice,
  body: (login: Login, config: Config) => Promise<number>
): Promise<number> {
  const config = loadConfigOrSay(ctx);
  if (!config) return 2;
  const hosts = resolver(ctx, config);
  const provider = await pickProvider(ctx, config, hosts, choice);
  if (typeof provider !== 'string') return fail(ctx, provider.error, provider.hints);
  return withLogin(ctx, config, hosts, provider, (login) => body(login, config));
}

function localLogin(ctx: Ctx, provider: Reachable): Login | null {
  const user = ctx.env.USER ?? ctx.username;
  const home = ctx.paths.home;
  const base = provider === 'dtu' ? `${DTU.projectDir}/${user}` : home;
  if (provider === 'dtu' && !existsSync(base)) {
    fail(ctx, `${base} does not exist.`, [NO_PROJECT_DIR]);
    return null;
  }
  const dir = join(ctx.env.XDG_CACHE_HOME || join(home, '.cache'), 'aicentre');
  mkdirSync(dir, { recursive: true });
  const file = join(dir, `session-${ctx.version}.sh`);
  writeFileSync(file, launcher);
  return {
    provider,
    master: new LocalShell(ctx),
    local: true,
    alias: 'localhost',
    user,
    home,
    base,
    data: `${base}/.aicentre`,
    launcher: file
  };
}

function setupCommand(name: string): string {
  return loginShell(
    [
      `d="\${XDG_CACHE_HOME:-$HOME/.cache}/aicentre" && mkdir -p "$d" && cat > "$d/${name}"`,
      `echo "@@FILE $d/${name}" && echo "@@USER $USER" && echo "@@HOME $HOME"`,
      `${VERSION_PROBE} && { ${PATH_PROBE}; }`,
      `{ [ -d "${DTU.projectDir}/$USER" ] && echo "@@P1 yes"; true; }`
    ].join(' && ')
  );
}

export async function openLogin(
  ctx: Ctx,
  config: Config,
  provider: Reachable,
  hosts = resolver(ctx, config)
): Promise<Login | null> {
  if (here(ctx) === provider) return localLogin(ctx, provider);

  const alias = await hosts.alias(provider);
  if (!alias) {
    fail(
      ctx,
      `no Host in ~/.ssh/config points at ${LOGIN[provider]} (${CLUSTER_NAMES[provider]}).`,
      ['aicentre check --all shows what to add.']
    );
    return null;
  }

  const socket = sharedSocket(ctx, alias);
  if (!(await isOpen(ctx, alias, socket))) {
    ctx.out.write(
      `Logging in to ${alias}. The login stays open for ${KEEP.replace('m', ' min')}, so the next commands do not ask again.\n`
    );
  }
  const master = new Master(alias, socket, ctx);
  const started = await master.start();
  if (!started.ok) {
    const f = classifySshFailure(
      started.stderr,
      providerContext(provider, alias, { user: config.gefion?.user })
    );
    fail(ctx, `cannot connect to ${alias}: ${f.message}`, f.hint);
    await master.close();
    return null;
  }

  const name = `session-${ctx.version}.sh`;
  const r = await master.run(setupCommand(name), { input: launcher, timeoutMs: 30_000 });
  const tags = parseTags(r.stdout);
  if (r.code !== 0 || !tags.FILE || !tags.USER || !tags.HOME) {
    fail(ctx, `the login node did not answer as expected: ${r.stderr.trim() || `exit ${r.code}`}`);
    await master.close();
    return null;
  }
  if (provider === 'dtu' && !tags.P1) {
    fail(ctx, `${DTU.projectDir}/${tags.USER} does not exist.`, [NO_PROJECT_DIR]);
    await master.close();
    return null;
  }

  const probe = {
    there: tags.AICENTRE ?? '',
    binOnPath: tags.BINPATH === 'yes',
    shell: tags.SHELL ?? ''
  };
  await sync(ctx, config, master, provider, alias, probe, makeStyle(ctx.env, ctx.out));
  const base = provider === 'dtu' ? `${DTU.projectDir}/${tags.USER}` : tags.HOME;
  return {
    provider,
    master,
    local: false,
    alias,
    user: tags.USER,
    home: tags.HOME,
    base,
    data: `${base}/.aicentre`,
    launcher: tags.FILE
  };
}

export function remotePath(login: Pick<Login, 'home' | 'base'>, path: string): string {
  if (!path) return login.base;
  if (path === '~') return login.home;
  if (path.startsWith('~/')) return `${login.home}/${path.slice(2)}`;
  if (path.startsWith('/')) return path;
  return `${login.base}/${path}`;
}
