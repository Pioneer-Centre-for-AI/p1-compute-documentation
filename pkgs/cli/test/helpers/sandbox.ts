import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import type { Ctx, Env } from '../../src/context';
import { resolvePaths } from '../../src/paths';

export type Sandbox = {
  root: string;
  home: string;
  fake: string;
  env: Env;
  out: Buffer;
  err: Buffer;
  ctx(overrides?: Partial<Ctx>): Ctx;
  file(path: string, text: string, mode?: number): string;
  answer(name: string, text: string): void;
  read(name: string): string;
  done(): void;
};

export class Buffer {
  text = '';
  isTTY = false;
  columns = 100;
  write(s: string) {
    this.text += s;
    return true;
  }
}

const REAL_TOOLS = ['sh', 'bash', 'tar', 'gzip', 'cat', 'env', 'sleep', 'touch', 'rm', 'mkdir'];

const FAKE_SSH = `#!/bin/sh
A="$FAKE/answers"
echo "$*" >> "$FAKE/calls"

case " $* " in
  *" ControlMaster=yes "*) touch "$FAKE/master.up"; exit 0 ;;
  *" -O check "*) [ -f "$FAKE/master.up" ]; exit $? ;;
  *" -O cancel "*) echo "$*" >> "$FAKE/cancel"; exit 0 ;;
  *" -O forward "*) exit 0 ;;
  *" -O exit "*) rm -f "$FAKE/master.up"; exit 0 ;;
esac
for last; do :; done
rec() { printf '%s\\n' "$last" >> "$FAKE/$1"; }
hold() { if [ -f "$A/$1" ]; then trap 'exit 143' TERM; while :; do sleep 0.1; done; fi; }
case "$last" in
  *"sync.lock"*) rec sync; exit 0 ;;
  *"cat > "*)
    cat > /dev/null
    printf '@@FILE /home/s123456/.cache/aicentre/session.sh\\n@@USER s123456\\n@@HOME /home/s123456\\n'
    printf '@@AICENTRE\\n@@BINPATH yes\\n@@SHELL bash\\n'
    [ -f "$A/no-p1" ] || echo '@@P1 yes'
    exit 0 ;;
  *"bkill"*) rec bkill; echo "Job <123> is being terminated"; exit 0 ;;
  *"scancel"*) rec scancel; exit 0 ;;
  *"bjobs"*)
    if [ -f "$A/needs-login" ] && [ ! -f "$FAKE/master.up" ]; then echo "s123456@login9.hpc.dtu.dk: Permission denied (password)." >&2; exit 255; fi
    cat "$A/bjobs" 2>/dev/null; cat "$A/bjobs-stderr" >&2 2>/dev/null; exit "$(cat "$A/bjobs.code" 2>/dev/null || echo 0)" ;;
  *"squeue"*) cat "$A/squeue" 2>/dev/null; cat "$A/squeue-stderr" >&2 2>/dev/null; exit "$(cat "$A/squeue.code" 2>/dev/null || echo 0)" ;;
  *"sbatch"*) rec sbatch; cat "$A/sbatch.out" 2>/dev/null; exit "$(cat "$A/sbatch.code" 2>/dev/null || echo 0)" ;;
  *"bsub "*" < "*) rec submit; cat "$A/submit.out" 2>/dev/null; exit "$(cat "$A/submit.code" 2>/dev/null || echo 0)" ;;
  *"bsub"*)
    rec bsub
    cat "$A/bsub.out" 2>/dev/null
    hold bsub.hold
    exit "$(cat "$A/bsub.code" 2>/dev/null || echo 0)" ;;
  *"session-"*|*"session.sh"*)
    rec login
    cat "$A/login.out" 2>/dev/null
    hold login.hold
    exit "$(cat "$A/login.code" 2>/dev/null || echo 0)" ;;
  *"mkdir -p "*) rec mkdir; exit 0 ;;
  "kill "*) rec kill; exit 0 ;;
esac

case "$1" in
  -V) cat "$A/ssh-V" >&2 2>/dev/null || echo "OpenSSH_9.8p1, LibreSSL 3.3.6" >&2; exit 0 ;;
  -Q) cat "$A/ssh-Q" 2>/dev/null || printf 'ssh-ed25519\\nssh-rsa\\n'; exit 0 ;;
  -G)
    if [ -f "$A/ssh-G-error" ]; then cat "$A/ssh-G-error" >&2; exit 255; fi
    if [ -f "$A/ssh-G-$2" ]; then cat "$A/ssh-G-$2"; exit 0; fi
    printf 'host %s\\nhostname %s\\nuser tester\\nport 22\\nidentityfile ~/.ssh/id_ed25519\\nuserknownhostsfile %s/.ssh/known_hosts\\nglobalknownhostsfile /etc/ssh/ssh_known_hosts\\n' "$2" "$2" "$HOME"
    exit 0 ;;
esac
for last; do :; done
for arg; do [ "$arg" = "$last" ] && break; alias="$arg"; done
cat "$A/probe-$alias.stdout" 2>/dev/null
cat "$A/probe-$alias.stderr" >&2 2>/dev/null
exit "$(cat "$A/probe-$alias.code" 2>/dev/null || echo 255)"
`;

const FAKE_SSH_KEYGEN = `#!/bin/sh
A="$FAKE/answers"
case "$1" in
  -F) [ -f "$A/known-$2" ] && { echo "# Host $2 found"; exit 0; }; exit 1 ;;
  -L) [ -f "$A/cert-valid" ] || exit 255; echo "$3:"; echo "        Valid: $(cat "$A/cert-valid")"; exit 0 ;;
esac
exit 1
`;

const FAKE_SSH_ADD = `#!/bin/sh
[ -f "$FAKE/answers/agent" ] && { cat "$FAKE/answers/agent"; exit 0; }
echo "The agent has no identities." ; exit 1
`;

const FAKE_TSH = `#!/bin/sh
A="$FAKE/answers"
case "$1 $2" in
  "version --client") echo '{"version":"18.10.0"}'; exit 0 ;;
  "status --client") cat "$A/tsh-status" 2>/dev/null || echo "ERROR: Not logged in." >&2; exit 0 ;;
esac
exit 1
`;

export function sandbox(): Sandbox {
  const root = mkdtempSync(join(tmpdir(), 'aicentre-test-'));
  const home = join(root, 'home');
  const fake = join(root, 'fake');
  const bin = join(fake, 'bin');
  const tools = join(fake, 'tools');
  for (const dir of [home, bin, tools, join(fake, 'answers'), join(root, 'run')])
    mkdirSync(dir, { recursive: true, mode: 0o700 });

  const script = (name: string, text: string) => {
    writeFileSync(join(bin, name), text);
    chmodSync(join(bin, name), 0o755);
  };
  script('ssh', FAKE_SSH);
  script('ssh-keygen', FAKE_SSH_KEYGEN);
  script('ssh-add', FAKE_SSH_ADD);
  script('tsh', FAKE_TSH);
  for (const tool of REAL_TOOLS) {
    const real = Bun.which(tool);
    if (real) symlinkSync(real, join(tools, tool));
  }

  const env: Env = {
    HOME: home,
    PATH: `${bin}:${tools}`,
    FAKE: fake,
    XDG_RUNTIME_DIR: join(root, 'run'),
    TERM: 'dumb',
    USER: 'tester'
  };
  const out = new Buffer();
  const err = new Buffer();

  return {
    root,
    home,
    fake,
    env,
    out,
    err,
    ctx(overrides = {}) {
      const e = overrides.env ?? env;
      return {
        env: e,
        paths: resolvePaths(e, process.getuid!()),
        uid: process.getuid!(),
        username: 'tester',
        out,
        err,
        interactive: false,
        execPath: '/nonexistent/aicentre',
        compiled: true,
        version: '0.1.0',
        platform: { os: 'linux', arch: 'x64' },
        ...overrides
      };
    },
    file(path, text, mode = 0o600) {
      const full = path.startsWith('/') ? path : join(home, path);
      mkdirSync(join(full, '..'), { recursive: true, mode: 0o700 });
      writeFileSync(full, text);
      chmodSync(full, mode);
      return full;
    },
    answer(name, text) {
      writeFileSync(join(fake, 'answers', name), text);
    },
    read(name) {
      return existsSync(join(fake, name)) ? readFileSync(join(fake, name), 'utf8') : '';
    },
    done() {
      if (!root.startsWith(tmpdir())) throw new Error(`refusing to remove ${root}`);
      rmSync(root, { recursive: true, force: true });
    }
  };
}

export function sshG(fields: Record<string, string | string[]>): string {
  return (
    Object.entries(fields)
      .flatMap(([k, v]) => (Array.isArray(v) ? v : [v]).map((x) => `${k} ${x}`))
      .join('\n') + '\n'
  );
}

export function dtuHost(sb: Sandbox): void {
  sb.answer(
    'ssh-G-dtu',
    sshG({ host: 'dtu', hostname: 'login9.hpc.dtu.dk', user: 's123456', port: '22' })
  );
}

export function gefionReady(sb: Sandbox, validTo: Date): void {
  const tsh = `${sb.home}/.tsh`;
  const cert = `${tsh}/keys/secureremote.dcai.dk/jdoe-ssh/secureremote.dcai.dk-cert.pub`;
  sb.file('.tsh/secureremote.dcai.dk.yaml', 'profile');
  sb.file(cert, 'cert');
  sb.file('.tsh/known_hosts', '@cert-authority *.secureremote.dcai.dk ssh-rsa AAAA\n');
  sb.answer(
    'ssh-G-gefion',
    sshG({
      host: 'gefion',
      hostname: 'slurm-login-01.hpc.ite.dcai.dk',
      user: 'jdoe',
      port: '3022',
      proxycommand:
        'tsh proxy ssh --cluster=secureremote.dcai.dk --proxy=secureremote.dcai.dk:443 %r@%h:%p',
      certificatefile: cert,
      identityfile: '~/.tsh/keys/secureremote.dcai.dk/jdoe',
      userknownhostsfile: `${tsh}/known_hosts`
    })
  );
  const local = new Date(validTo.getTime() - validTo.getTimezoneOffset() * 60_000);
  sb.answer('cert-valid', `from 2026-01-01T00:00:00 to ${local.toISOString().slice(0, 19)}`);
}
