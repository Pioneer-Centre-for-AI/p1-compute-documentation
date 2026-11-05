# Check

`aicentre check` checks this machine first, then each provider, all
providers at the same time (`src/check/run.ts`). Without names, a provider is
checked when it is [in use](03-providers-and-login.md#which-providers-a-machine-uses).
The rest show as not set up, which is not a failure: most people use one or
two. `-p` checks only the providers named, and missing setup there is a
failure. `--all` checks every provider and cannot be combined with `-p`.
The network path is part of every check: a TCP connect with no login, so it
cannot prompt. When it fails it is a warning, because the setup can be right
on a network that blocks the way. login9 takes SSH from the internet, so the
hint for DTU is the machine's connection or a network that blocks port 22, and
the DTU VPN as a way around one. `--connect`
adds the login steps, and then a failed network path is a failure. `--json` is
for scripts.

Each group is a list of steps with a stable id (`ssh-config`, `dtu.key`, ...)
for `--json`. A step reports `ok`, `warn`, `fail`, `skip` or `info`, a
one-line detail, and hint lines under a warning or failure. A step that cannot
run because an earlier one failed is recorded as skipped with the reason. A
group's status is its worst step.

## Probes

The checks ask the tools rather than parse files by hand
(`src/check/probes/`).

- `ssh.ts`: `ssh -V` (it prints to stderr) for the version, and `ssh -G` for
  the configuration ssh would use for a host after every Include and Match.
  With no matching Host block, `ssh -G` echoes the alias back as the hostname.
  Anything else means a block, a Match or a CanonicalizeHostname rule points
  it somewhere. `userknownhostsfile` packs several paths into one line. Paths
  are expanded for `~` and the ssh_config tokens that matter for files.
- `files.ts`: the permission rules OpenSSH applies, checked before ssh gets to
  complain. `readconf.c` refuses a config file that is not owned by the user
  or root, or that others can write. `stat` follows symlinks, as ssh does, so
  a root-owned link into the Nix store passes. `authfile.c` ignores a private
  key the user owns that the group or others can read or write. `~/.ssh`
  should be `0700`.
- `knownhosts.ts`: `ssh-keygen -F`, which understands hashed entries. It exits
  0 on a match, 1 without one, and 255 for a missing file. A host on a port
  other than 22 is stored as `[host]:port`.
- `net.ts`: a name lookup, then a TCP connect, with a 3 second limit (5 with
  `--connect`). It runs before ssh, so a VPN or firewall problem is not
  mistaken for a key problem. Checks call it as `ctx.tcp`, which the test
  sandbox replaces, so tests never touch the network.
- `tsh.ts`: Teleport, read from files only. `tsh version` and `tsh status`
  contact the proxy unless given `--client`, which tsh before 18.7 lacks. So
  the profile is `~/.tsh/<proxy>.yaml`, the users are the
  `keys/<proxy>/<user>-ssh/` directories, and the login's validity comes from
  `ssh-keygen -L` on the certificate. That prints local time without a zone,
  and `forever` on either side means no limit. `tsh version --client` failing
  on an old tsh still means it is installed.
- `connect.ts`: one real login, the way `ssh <alias>` would make it, with
  `BatchMode=yes`, `ConnectTimeout=10`, `ConnectionAttempts=1`,
  `ControlMaster=no`, `RequestTTY=no`, `ClearAllForwardings=yes`, and
  `RemoteCommand=none` on OpenSSH 7.6 or newer (older ssh stops on an option
  it does not know). BatchMode turns every question into a failure explained
  from stderr. `ControlMaster=no` leaves no shared connection behind, but a
  shared login the user already has open is used, which is the only way past
  a password prompt. The probe has 25 seconds.

## The failure classifier

`classifySshFailure` turns ssh's stderr into a message and hints. The same
classifier serves `check`, [login](03-providers-and-login.md#connecting) and
`list jobs`. Rules are tried in order:

| Kind | Means |
| --- | --- |
| `teleport` | Teleport refused: expired, not logged in, tsh missing. Only counts when the Host goes through tsh. |
| `dns` | The host name does not resolve |
| `timeout` | No answer. For DTU the hint is the machine's connection or a network that blocks port 22, or for Gefion that Teleport answered but the login node did not |
| `refused`, `reset` | Refused, or closed before login |
| `host-key-changed` | Remove the old key only if the provider announced a new one |
| `host-key-unknown` | A warning: connect once by hand and answer yes |
| `unprotected-key` | ssh ignored a key others can read |
| `too-many-keys` | Add `IdentitiesOnly yes` |
| `password` | The key was accepted and the login node also wants a password, a second factor BatchMode cannot give. DTU asks for it off its own network. Any interactive command logs in once. |
| `key-refused` | Copy the key with `ssh-copy-id`, or fix Teleport |

Anything else reports ssh's last line and suggests `ssh -v`.

## Steps

This machine (`local.ts`): OpenSSH 6.8 or newer, for `ssh -G`; `~/.ssh`
private; `~/.ssh/config` readable by ssh. For the last, `ssh -G` runs on a
host nobody configures, `aicentre-check.invalid`, because ssh still reads
every line on the way and syntax errors surface.

DTU (`dtu.ts`): the Host block points at `login9.hpc.dtu.dk`; the User is not
a `<placeholder>` and agrees with `config.toml`; a usable key on disk or in
the agent; the host key is known; the network path, unless a ProxyJump or
ProxyCommand means the login node is not reached directly. With `--connect`:
the login; `/dtu/p1/<user>`; the aicentre version there.

Gefion (`gefion.ts`): tsh installed; the Host block has HostName, port 3022
and a `tsh proxy ssh` ProxyCommand; CertificateFile, IdentityFile and
UserKnownHostsFile point at the files `tsh login` writes
(`~/.tsh/keys/<proxy>/<user>-ssh/<proxy>-cert.pub`,
`~/.tsh/keys/<proxy>/<user>` and `~/.tsh/known_hosts`); the certificate is
valid, with a warning under an hour left. A login lasts about 12 hours; port
443 on the proxy answers. With `--connect`: the login and the aicentre version
there.

NGC (`ngc.ts`) is air-gapped. People work on it through its remote desktop,
so there is nothing on a laptop to check.

When a Host block is missing or wrong, the hint prints the block to add from
`hostBlock` in `clusters.ts`, with the user filled in when known.

## Output

`src/ui/report.ts` has three reporters: a live checklist on a terminal, plain
blocks when output goes to a file or pipe, and JSON. All print groups in a
fixed order (this machine, DTU, Gefion, NGC), even though providers are
checked at the same time. The live checklist prints a finished group once
every group before it has finished. Ctrl-C puts the terminal back the way it
was.
