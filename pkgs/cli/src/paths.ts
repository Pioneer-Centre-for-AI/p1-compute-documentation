import { lstatSync, mkdirSync } from 'node:fs';
import { isAbsolute, join } from 'node:path';
import type { Env } from './context';

export type Paths = {
  home: string;
  binDir: string;
  binLink: string;
  data: string;
  versions: string;
  current: string;
  manifest: string;
  config: string;
  configFile: string;
  state: string;
  cache: string;
  runtime: string;
};

function xdg(env: Env, name: string, fallback: string): string {
  const value = env[name];
  return value && isAbsolute(value) ? value : fallback;
}

export function resolvePaths(env: Env, uid: number): Paths {
  const home = env.HOME;
  if (!home || !isAbsolute(home)) throw new Error('HOME is not set to an absolute path');

  const data = join(xdg(env, 'XDG_DATA_HOME', join(home, '.local/share')), 'aicentre');
  const config = join(xdg(env, 'XDG_CONFIG_HOME', join(home, '.config')), 'aicentre');
  const binDir = xdg(env, 'XDG_BIN_HOME', join(home, '.local/bin'));
  const runtimeBase = env.XDG_RUNTIME_DIR;
  const runtime =
    runtimeBase && isAbsolute(runtimeBase)
      ? join(runtimeBase, 'aicentre')
      : join(xdg(env, 'TMPDIR', '/tmp'), `aicentre-${uid}`);

  return {
    home,
    binDir,
    binLink: join(binDir, 'aicentre'),
    data,
    versions: join(data, 'versions'),
    current: join(data, 'current'),
    manifest: join(data, 'install.json'),
    config,
    configFile: join(config, 'config.toml'),
    state: join(xdg(env, 'XDG_STATE_HOME', join(home, '.local/state')), 'aicentre'),
    cache: join(xdg(env, 'XDG_CACHE_HOME', join(home, '.cache')), 'aicentre'),
    runtime
  };
}

export function ensureRuntimeDir(paths: Paths, uid: number): string {
  mkdirSync(paths.runtime, { recursive: true, mode: 0o700 });
  const st = lstatSync(paths.runtime);
  if (st.isSymbolicLink() || !st.isDirectory())
    throw new Error(`${paths.runtime} is not a directory`);
  if (st.uid !== uid) throw new Error(`${paths.runtime} belongs to another user`);
  if ((st.mode & 0o077) !== 0)
    throw new Error(
      `${paths.runtime} is open to other users (mode ${(st.mode & 0o777).toString(8)})`
    );
  return paths.runtime;
}

export function tilde(path: string, home: string): string {
  return path === home ? '~' : path.startsWith(home + '/') ? '~' + path.slice(home.length) : path;
}
