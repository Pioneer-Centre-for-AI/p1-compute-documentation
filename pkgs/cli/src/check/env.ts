import type { Config } from '../config';
import type { Ctx } from '../context';
import type { ClusterId } from './clusters';
import { expandPath, type SshGResult, type SshVersion } from './probes/ssh';

export type Machine = {
  ssh: SshVersion | null;
  sshUsable: boolean;
};

export type CheckEnv = {
  ctx: Ctx;
  config: Config;
  connect: boolean;
  explicit: boolean;
  machine: Machine;
  sshG(alias: string): Promise<SshGResult>;
  alias(id: ClusterId): Promise<string | null>;
};

export function expander(
  c: CheckEnv,
  remoteUser?: string,
  host?: string
): (path: string) => string {
  const t = { home: c.ctx.paths.home, localUser: c.ctx.username, remoteUser, host, uid: c.ctx.uid };
  return (path) => expandPath(path, t);
}

export function blockHint(intro: string, block: string[]): string[] {
  return [intro, '', ...block.map((line) => `  ${line}`)];
}

export function cliThere(
  version: string,
  there: string | undefined
): { status: 'ok' | 'info'; detail: string } {
  if (!there)
    return {
      status: 'info',
      detail: 'not installed there yet; the next session or job installs it'
    };
  if (there === version) return { status: 'ok', detail: `${there}, the same as here` };
  return {
    status: 'info',
    detail: `${there} there, ${version} here; the next session or job brings it in line`
  };
}
