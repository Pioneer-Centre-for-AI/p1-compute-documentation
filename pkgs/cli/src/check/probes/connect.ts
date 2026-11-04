import { run } from '../../exec';
import type { Env } from '../../context';
import { atLeast, type SshVersion } from './ssh';
import { DTU, DTU_VPN_HINT, GEFION, tshLogin } from '../clusters';

export function probeArgs(
  alias: string,
  version: SshVersion | null,
  script: string,
  socket?: string
): string[] {
  const opts = [
    'BatchMode=yes',
    'ConnectTimeout=10',
    'ConnectionAttempts=1',
    'ControlMaster=no',
    'RequestTTY=no',
    'ClearAllForwardings=yes'
  ];
  if (version && atLeast(version, 7, 6)) opts.push('RemoteCommand=none');
  const shared = socket ? ['-S', socket] : [];
  return [...opts.flatMap((o) => ['-o', o]), ...shared, '-T', '-n', alias, `sh -c '${script}'`];
}

export type Failure = { kind: string; status: 'fail' | 'warn'; message: string; hint: string[] };

export type FailureContext = {
  alias: string;
  host: string;
  viaTsh: boolean;
  timedOut: boolean;
  timeoutHint: string[];
  teleportHint: string[];
};

export function providerContext(
  provider: 'dtu' | 'gefion',
  alias: string,
  opts: { user?: string; host?: string; timedOut?: boolean } = {}
): FailureContext {
  const gefion = provider === 'gefion';
  return {
    alias,
    host: opts.host ?? (gefion ? GEFION.login : DTU.login),
    viaTsh: gefion,
    timedOut: opts.timedOut ?? false,
    timeoutHint: gefion
      ? [
          'Teleport answered, but the login node did not. Try again in a minute, or ask DCAI support (support@dcai.dk).'
        ]
      : [DTU_VPN_HINT],
    teleportHint: gefion ? [tshLogin(opts.user || undefined)] : []
  };
}

type Rule = {
  kind: string;
  test: RegExp;
  status?: 'fail' | 'warn';
  message: string;
  hint: (c: FailureContext) => string[];
};

const RULES: Rule[] = [
  {
    kind: 'teleport',
    test: /ERROR:.*(expired|not logged in|access denied)|exec: "?tsh"?: not found|tsh: (command )?not found|Connection closed by UNKNOWN port 65535/i,
    message: 'Teleport refused the connection',
    hint: (c) => c.teleportHint
  },
  {
    kind: 'dns',
    test: /Could not resolve hostname|Name or service not known|nodename nor servname provided|Temporary failure in name resolution/i,
    message: 'the host name does not resolve',
    hint: (c) => [`Check the HostName in Host ${c.alias}, and that this machine is online.`]
  },
  {
    kind: 'timeout',
    test: /(Connection|Operation) timed out|timed out during banner exchange/i,
    message: 'the login node did not answer',
    hint: (c) => c.timeoutHint
  },
  {
    kind: 'refused',
    test: /Connection refused|No route to host|Network is unreachable/i,
    message: 'the connection was refused',
    hint: (c) => c.timeoutHint
  },
  {
    kind: 'reset',
    test: /kex_exchange_identification|Connection reset by peer|Connection closed by remote host/i,
    message: 'the server closed the connection before login',
    hint: (c) => c.timeoutHint
  },
  {
    kind: 'host-key-changed',
    test: /REMOTE HOST IDENTIFICATION HAS CHANGED|Host key for .* has changed/i,
    message: 'the host key changed since you last connected',
    hint: (c) => [
      'If the provider announced a new key, remove the old one and connect again:',
      `  ssh-keygen -R ${c.host}`,
      'If it did not, stop and ask the compute coordinator.'
    ]
  },
  {
    kind: 'host-key-unknown',
    test: /Host key verification failed/i,
    status: 'warn',
    message: 'this machine has not seen the host key yet',
    hint: (c) => [`Connect once by hand and answer yes: ssh ${c.alias}`]
  },
  {
    kind: 'unprotected-key',
    test: /UNPROTECTED PRIVATE KEY FILE|bad permissions/i,
    message: 'ssh ignored a key because others can read it',
    hint: () => ['chmod 600 the key file that ssh named, then try again.']
  },
  {
    kind: 'too-many-keys',
    test: /Too many authentication failures/i,
    message: 'ssh offered too many keys before the right one',
    hint: (c) => [`Add "IdentitiesOnly yes" and the right IdentityFile to Host ${c.alias}.`]
  },
  {
    kind: 'password',
    test: /Permission denied \((?![^)]*publickey)[^)]*(password|keyboard-interactive)[^)]*\)/i,
    message: 'your key was accepted, and the login node also asks for your password',
    hint: () => [
      'In a terminal, aicentre asks for it once and keeps the login open for 30 min.',
      'aicentre list jobs, or any session or batch command, does that.'
    ]
  },
  {
    kind: 'key-refused',
    test: /Permission denied \([^)]*\)/i,
    message: 'the login node did not accept your key',
    hint: (c) =>
      c.viaTsh
        ? c.teleportHint
        : [
            `Check User in Host ${c.alias}, then copy your public key to the login node:`,
            `  ssh-copy-id ${c.alias}`,
            'If your key has a passphrase, load it into the agent first: ssh-add'
          ]
  }
];

export function classifySshFailure(stderr: string, c: FailureContext): Failure {
  if (c.timedOut) {
    return {
      kind: 'timeout',
      status: 'fail',
      message: 'ssh gave up after 25 s',
      hint: c.viaTsh ? c.teleportHint : c.timeoutHint
    };
  }
  for (const rule of RULES) {
    if (rule.kind === 'teleport' && !c.viaTsh) continue;
    if (rule.test.test(stderr))
      return {
        kind: rule.kind,
        status: rule.status ?? 'fail',
        message: rule.message,
        hint: rule.hint(c)
      };
  }
  const last = stderr.trim().split('\n').filter(Boolean).at(-1);
  return {
    kind: 'other',
    status: 'fail',
    message: last ?? 'ssh failed without saying why',
    hint: [`Run ssh -v ${c.alias} to see where it stops.`]
  };
}

export function parseTags(stdout: string): Record<string, string> {
  const tags: Record<string, string> = {};
  for (const line of stdout.split('\n')) {
    const m = /^@@([A-Z0-9]+) ?(.*)$/.exec(line.trim());
    if (m) tags[m[1]!] = m[2] ?? '';
  }
  return tags;
}

export type ProbeResult =
  { ok: true; tags: Record<string, string> } | { ok: false; failure: Failure };

export async function sshProbe(
  version: SshVersion | null,
  script: string,
  c: Omit<FailureContext, 'timedOut'>,
  env: Env,
  socket?: string
): Promise<ProbeResult> {
  const r = await run('ssh', probeArgs(c.alias, version, script, socket), {
    env,
    timeoutMs: 25_000,
    detached: true
  });
  const tags = parseTags(r.stdout);
  if (r.code === 0 && tags.HOST !== undefined) return { ok: true, tags };
  return { ok: false, failure: classifySshFailure(r.stderr, { ...c, timedOut: r.timedOut }) };
}
