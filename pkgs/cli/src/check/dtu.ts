import { existsSync } from 'node:fs';
import { run } from '../exec';
import { tilde } from '../paths';
import type { Group } from './types';
import { blockHint, cliThere, expander, type CheckEnv } from './env';
import { VERSION_PROBE } from '../session/sync';
import { isOpen, sharedSocket } from '../remote/master';
import { DTU, DTU_VPN_HINT, hostBlock } from './clusters';
import { privateKeyProblem, statOrNull } from './probes/files';
import { knownHost } from './probes/knownhosts';
import { tcpProbe } from './probes/net';
import { all, get, isConfigured, words, type SshConfig } from './probes/ssh';
import { providerContext, sshProbe } from './probes/connect';

export async function checkDtu(g: Group, c: CheckEnv): Promise<void> {
  const alias = (await c.alias('dtu')) ?? DTU.alias;
  const wantUser = c.config.dtu?.user;
  const home = c.ctx.paths.home;

  if (!c.machine.sshUsable) {
    await g.skip('host', 'Host in ~/.ssh/config', 'needs a working ssh');
    return;
  }

  let cfg: SshConfig | undefined;
  const host = await g.step('host', 'Host in ~/.ssh/config', async () => {
    const res = await c.sshG(alias);
    if (!res.ok) return { status: 'fail', detail: res.error };
    if (!isConfigured(res.cfg, alias)) {
      return {
        status: 'fail',
        detail: `none points at ${DTU.login}`,
        hint: blockHint(
          'Add this to ~/.ssh/config. The Host name is yours to choose:',
          hostBlock('dtu', alias, wantUser)
        )
      };
    }
    cfg = res.cfg;
    const hostname = get(cfg, 'hostname');
    if (hostname !== DTU.login) {
      return { status: 'warn', detail: `points at ${hostname}; P1 uses ${DTU.login}` };
    }
    return { status: 'ok', detail: `${alias} → ${hostname}` };
  });
  if (host.status === 'fail' || !cfg) return;
  const conf = cfg;
  const hostname = get(conf, 'hostname') ?? DTU.login;
  const port = Number(get(conf, 'port') ?? 22);
  const user = get(conf, 'user') ?? '';
  const expand = expander(c, user, hostname);

  await g.step('user', 'User', () => {
    if (/[<>]/.test(user)) {
      return {
        status: 'fail',
        detail: `still the placeholder ${user}`,
        hint: [`Set User to your DTU username in Host ${alias}.`]
      };
    }
    if (wantUser && user !== wantUser) {
      return {
        status: 'fail',
        detail: `${user}, but config.toml says ${wantUser}`,
        hint: [`Set "User ${wantUser}" in Host ${alias}, or fix config.toml.`]
      };
    }
    if (!wantUser && user === c.ctx.username) {
      return { status: 'ok', detail: `${user}, the same name as on this machine` };
    }
    return { status: 'ok', detail: user };
  });

  await g.step('key', 'SSH key', async () => {
    const keys = all(conf, 'identityfile')
      .map(expand)
      .filter((path) => existsSync(path));
    for (const key of keys) {
      const st = statOrNull(key);
      const problem = st && privateKeyProblem(st, c.ctx.uid);
      if (problem)
        return {
          status: 'fail',
          detail: `${tilde(key, home)}: ${problem}`,
          hint: [`chmod 600 ${tilde(key, home)}`]
        };
    }
    const agent = await run('ssh-add', ['-l'], { env: c.ctx.env, timeoutMs: 5_000 });
    const agentKeys = agent.code === 0 ? agent.stdout.trim().split('\n').filter(Boolean).length : 0;
    if (keys.length === 0 && agentKeys === 0) {
      return {
        status: 'fail',
        detail: 'no key found',
        hint: [
          'Make one and copy it to the login node:',
          '  ssh-keygen -t ed25519',
          `  ssh-copy-id ${alias}`
        ]
      };
    }
    const parts = keys.map((k) => tilde(k, home));
    if (agentKeys) parts.push(`${agentKeys} in the agent`);
    return { status: 'ok', detail: parts.join(', ') };
  });

  await g.step('known-host', 'Host key', async () => {
    const files = [
      ...words(conf, 'userknownhostsfile'),
      ...words(conf, 'globalknownhostsfile')
    ].map(expand);
    const found = await knownHost(hostname, port, files, c.ctx.env);
    if (found) return { status: 'ok', detail: `known, in ${tilde(found, home)}` };
    return {
      status: 'warn',
      detail: 'not seen yet, so the first ssh asks you to confirm it',
      hint: [`ssh ${alias}`]
    };
  });

  if (!c.connect) return;

  const proxied = Boolean(get(conf, 'proxyjump') ?? get(conf, 'proxycommand'));
  if (!proxied) {
    const reach = await g.step('reach', `Network path to ${hostname}`, async () => {
      const r = await tcpProbe(hostname, port);
      return r.ok
        ? { status: 'ok', detail: `port ${port} answers` }
        : { status: 'fail', detail: r.message, hint: [DTU_VPN_HINT] };
    });
    if (reach.status === 'fail') {
      await g.skip('login', 'Login', 'needs the network path');
      return;
    }
  }

  let p1: string | undefined;
  let there: string | undefined;
  const login = await g.step('login', 'Login', async () => {
    const script = `printf "@@HOST %s\\n" "$(hostname)"; d=${DTU.projectDir}/$USER; [ -d "$d" ] && printf "@@P1 %s\\n" "$d"; ${VERSION_PROBE}; true`;
    const r = await sshProbe(
      c.machine.ssh,
      script,
      providerContext('dtu', alias, { host: hostname }),
      c.ctx.env,
      (await isOpen(c.ctx, alias)) ? sharedSocket(c.ctx, alias) : undefined
    );
    if (!r.ok) return { status: r.failure.status, detail: r.failure.message, hint: r.failure.hint };
    p1 = r.tags.P1;
    there = r.tags.AICENTRE;
    return { status: 'ok', detail: `reached ${r.tags.HOST}` };
  });
  if (login.status !== 'ok') return;

  await g.step('project', 'Project directory', () =>
    p1
      ? { status: 'ok', detail: p1 }
      : {
          status: 'fail',
          detail: `${DTU.projectDir}/${user} does not exist`,
          hint: ['Ask the compute coordinator to set up your P1 project directory.']
        }
  );
  await g.step('cli', 'aicentre there', () => cliThere(c.ctx.version, there || undefined));
}
