import { join } from 'node:path';
import type { Ctx } from '../context';
import type { Group } from './types';
import type { Machine } from './env';
import { configProblem, sshDirProblem, statOrNull } from './probes/files';
import { atLeast, sshG, sshVersion } from './probes/ssh';

export async function checkLocal(g: Group, ctx: Ctx): Promise<Machine> {
  const machine: Machine = { ssh: null, sshUsable: false };
  const home = ctx.paths.home;
  const sshDir = join(home, '.ssh');
  const sshConfig = join(sshDir, 'config');

  await g.step('ssh', 'OpenSSH client', async () => {
    const v = await sshVersion(ctx.env);
    if (!v) {
      return {
        status: 'fail',
        detail: 'ssh is not installed, or is not OpenSSH',
        hint: [
          ctx.platform.os === 'darwin'
            ? 'macOS ships OpenSSH; check that /usr/bin is on your PATH.'
            : 'Install OpenSSH with your package manager (openssh-client or openssh).'
        ]
      };
    }
    machine.ssh = v;
    if (!atLeast(v, 6, 8)) {
      return {
        status: 'fail',
        detail: `OpenSSH ${v.text} is too old; aicentre needs 6.8 or newer`,
        hint: ['Update OpenSSH with your package manager.']
      };
    }
    machine.sshUsable = true;
    return { status: 'ok', detail: `OpenSSH ${v.text}` };
  });

  await g.step('ssh-dir', '~/.ssh', () => {
    const st = statOrNull(sshDir);
    if (!st) return { status: 'warn', detail: 'does not exist yet', hint: ['mkdir -m 700 ~/.ssh'] };
    const problem = sshDirProblem(st, ctx.uid);
    if (problem) return { status: 'warn', detail: problem, hint: ['chmod 700 ~/.ssh'] };
    return { status: 'ok', detail: 'private to you' };
  });

  await g.step('ssh-config', '~/.ssh/config', async () => {
    const st = statOrNull(sshConfig);
    if (!st)
      return {
        status: 'warn',
        detail: 'does not exist yet; each provider below shows what to put in it'
      };
    const problem = configProblem(st, ctx.uid);
    if (problem) {
      return {
        status: 'fail',
        detail: `${problem}, so ssh refuses to read it`,
        hint: [
          st.uid === ctx.uid || st.uid === 0
            ? 'chmod 600 ~/.ssh/config'
            : `chown ${ctx.username || '$USER'} ~/.ssh/config`
        ]
      };
    }
    if (!machine.sshUsable) return { status: 'skip', detail: 'needs a working ssh to read' };
    const parsed = await sshG('aicentre-check.invalid', ctx.env);
    if (!parsed.ok)
      return { status: 'fail', detail: 'ssh cannot read it', hint: parsed.error.split('\n') };
    return { status: 'ok', detail: 'ssh reads it' };
  });

  return machine;
}
