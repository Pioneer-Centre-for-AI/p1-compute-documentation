import { existsSync } from 'node:fs';
import { join } from 'node:path';
import { tilde } from '../paths';
import type { Group } from './types';
import { blockHint, cliThere, expander, type CheckEnv } from './env';
import { VERSION_PROBE } from '../session/sync';
import { isOpen, sharedSocket } from '../remote/master';
import { GEFION, hostBlock, teleportCert, teleportKey, tshLogin } from './clusters';
import { tcpProbe } from './probes/net';
import { providerContext, sshProbe } from './probes/connect';
import { all, get, isConfigured, words, type SshConfig } from './probes/ssh';
import { certValidity, duration, profileUsers, tshHome, tshVersion } from './probes/tsh';

const HOUR = 3_600_000;

export async function checkGefion(g: Group, c: CheckEnv): Promise<void> {
  const alias = (await c.alias('gefion')) ?? GEFION.alias;
  const wantUser = c.config.gefion?.user;
  const home = c.ctx.paths.home;
  const tsh = tshHome(c.ctx.env, home);
  const guessUser = wantUser ?? profileUsers(tsh, GEFION.proxy)[0];

  await g.step('tsh', 'Teleport client (tsh)', async () => {
    const v = await tshVersion(c.ctx.env);
    if (v === 'missing') {
      return {
        status: 'fail',
        detail: 'not installed',
        hint: ['Install tsh: https://goteleport.com/docs/connect-your-client/tsh/']
      };
    }
    if (v === 'unknown') return { status: 'ok', detail: 'installed' };
    return { status: 'ok', detail: `tsh ${v.text}` };
  });

  if (!c.machine.sshUsable) {
    await g.skip('host', 'Host in ~/.ssh/config', 'needs a working ssh');
    return;
  }

  let cfg: SshConfig | undefined;
  const host = await g.step('host', 'Host in ~/.ssh/config', async () => {
    const res = await c.sshG(alias);
    if (!res.ok) return { status: 'fail', detail: res.error };
    const block = hostBlock('gefion', alias, guessUser);
    if (!isConfigured(res.cfg, alias)) {
      return {
        status: 'fail',
        detail: `none points at ${GEFION.login}`,
        hint: blockHint('Add this to ~/.ssh/config. The Host name is yours to choose:', block)
      };
    }
    cfg = res.cfg;
    const problems: string[] = [];
    const hostname = get(cfg, 'hostname');
    const port = Number(get(cfg, 'port') ?? 22);
    if (hostname !== GEFION.login) problems.push(`HostName is ${hostname}`);
    if (port !== GEFION.port) problems.push(`Port is ${port}, not ${GEFION.port}`);
    if (!/\btsh\b.*\bproxy\s+ssh\b/.test(get(cfg, 'proxycommand') ?? ''))
      problems.push('no tsh ProxyCommand');
    if (problems.length)
      return {
        status: 'fail',
        detail: problems.join('; '),
        hint: blockHint(`Compare Host ${alias} with:`, block)
      };
    return { status: 'ok', detail: `${alias} → ${hostname}:${port} through Teleport` };
  });
  if (host.status === 'fail' || !cfg) return;
  const conf = cfg;
  const user = get(conf, 'user') ?? '';
  const expand = expander(c, user, get(conf, 'hostname'));
  const cert = teleportCert(tsh, user);
  const key = teleportKey(tsh, user);
  const knownHosts = join(tsh, 'known_hosts');

  await g.step('paths', 'Teleport files in Host ' + alias, () => {
    if (/[<>]/.test(user)) {
      return {
        status: 'fail',
        detail: `User is still the placeholder ${user}`,
        hint: [`Set User to your Gefion username in Host ${alias}.`]
      };
    }
    if (wantUser && user !== wantUser) {
      return { status: 'fail', detail: `User is ${user}, but config.toml says ${wantUser}` };
    }
    const problems: string[] = [];
    if (!all(conf, 'certificatefile').map(expand).includes(cert))
      problems.push(`CertificateFile is not ${tilde(cert, home)}`);
    if (!all(conf, 'identityfile').map(expand).includes(key))
      problems.push(`IdentityFile is not ${tilde(key, home)}`);
    if (!words(conf, 'userknownhostsfile').map(expand).includes(knownHosts))
      problems.push(`UserKnownHostsFile is not ${tilde(knownHosts, home)}`);
    if (problems.length)
      return {
        status: 'fail',
        detail: problems.join('; '),
        hint: blockHint(`Compare Host ${alias} with:`, hostBlock('gefion', alias, user))
      };
    return { status: 'ok', detail: `for ${user}` };
  });

  let loggedIn = false;
  await g.step('login', 'Teleport login', async () => {
    const fix = [tshLogin(user), 'A login lasts about 12 hours.'];
    if (!existsSync(cert)) return { status: 'fail', detail: 'no certificate yet', hint: fix };
    const validity = await certValidity(cert, c.ctx.env);
    if (!validity) return { status: 'warn', detail: `cannot read ${tilde(cert, home)}`, hint: fix };
    const left = validity.to ? validity.to.getTime() - Date.now() : Infinity;
    if (left <= 0) return { status: 'fail', detail: `expired ${duration(left)} ago`, hint: fix };
    loggedIn = true;
    if (left < HOUR)
      return { status: 'warn', detail: `expires in ${duration(left)}`, hint: [tshLogin(user)] };
    return { status: 'ok', detail: left === Infinity ? 'valid' : `valid for ${duration(left)}` };
  });

  if (!c.connect) return;

  let there: string | null | undefined = null;
  const reach = await g.step('reach', `Network path to ${GEFION.proxy}`, async () => {
    const r = await tcpProbe(GEFION.proxy, GEFION.proxyPort);
    return r.ok
      ? { status: 'ok', detail: `port ${GEFION.proxyPort} answers` }
      : {
          status: 'fail',
          detail: r.message,
          hint: ['Teleport needs outbound HTTPS. Check that this machine is online.']
        };
  });

  if (reach.status === 'fail') await g.skip('ssh', 'Login', 'needs the network path');
  else if (!loggedIn) await g.skip('ssh', 'Login', 'needs a valid Teleport login');
  else {
    await g.step('ssh', 'Login', async () => {
      const r = await sshProbe(
        c.machine.ssh,
        `printf "@@HOST %s\\n" "$(hostname)"; ${VERSION_PROBE}`,
        providerContext('gefion', alias, { user }),
        c.ctx.env,
        (await isOpen(c.ctx, alias)) ? sharedSocket(c.ctx, alias) : undefined
      );
      if (!r.ok)
        return { status: r.failure.status, detail: r.failure.message, hint: r.failure.hint };
      there = r.tags.AICENTRE;
      return { status: 'ok', detail: `reached ${r.tags.HOST}` };
    });
    if (there !== null)
      await g.step('cli', 'aicentre there', () => cliThere(c.ctx.version, there || undefined));
  }
}
