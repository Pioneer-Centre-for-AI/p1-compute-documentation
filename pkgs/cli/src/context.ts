import { resolvePaths, type Paths } from './paths';
import { currentPlatform, type Platform } from './platform';
import { VERSION } from './version';

export type Env = Record<string, string | undefined>;

export type Output = {
  write(text: string): unknown;
  isTTY?: boolean;
  columns?: number;
};

export type Ctx = {
  env: Env;
  paths: Paths;
  uid: number;
  username: string;
  out: Output;
  err: Output;
  interactive: boolean;
  execPath: string;
  compiled: boolean;
  version: string;
  platform: Platform;
};

export function processCtx(): Ctx {
  const env = process.env as Env;
  const uid = process.getuid?.() ?? 0;
  return {
    env,
    paths: resolvePaths(env, uid),
    uid,
    username: env.USER ?? env.LOGNAME ?? '',
    out: process.stdout,
    err: process.stderr,
    interactive: Boolean(process.stdin.isTTY && process.stdout.isTTY),
    execPath: process.execPath,
    compiled: isCompiled(),
    version: VERSION,
    platform: currentPlatform()
  };
}

export function isCompiled(): boolean {
  const said = (Bun as { isStandaloneExecutable?: boolean }).isStandaloneExecutable;
  if (typeof said === 'boolean') return said;
  return (
    import.meta.url.includes('$bunfs') ||
    import.meta.url.includes('~BUN') ||
    import.meta.url.includes('%7EBUN')
  );
}
