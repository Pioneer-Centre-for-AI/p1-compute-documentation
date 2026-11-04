# Providers and login

P1 compute has three providers. `src/check/clusters.ts` holds what a working
setup looks like for each, taken from the provider pages under
`src/routes/providers/` and the gallery's `pkgs/gallery/rem-lgn/ssh_config`.
Change the three together.

| Id | Login | How |
| --- | --- | --- |
| `dtu` | `login9.hpc.dtu.dk`, port 22 | Plain OpenSSH with a key the user copied there. DTU asks for the password after the key. Outside DTU's network it needs the DTU VPN. The project directory is `/dtu/p1/<user>`. |
| `gefion` | `slurm-login-01.hpc.ite.dcai.dk`, port 3022 | ssh through Teleport: `tsh login` writes a short-lived certificate under `~/.tsh` (or `$TELEPORT_HOME`), and the Host block has a `tsh proxy ssh` ProxyCommand. The proxy is `secureremote.dcai.dk:443`. |
| `ngc` | none | Air-gapped. People work on it through its remote desktop, so nothing on a laptop can reach it. Commands that need a login node refuse it. |

The code calls DTU and Gefion `Reachable`. Table columns use the short names
DTU, Gefion and NGC, since "P1 ... HPC" around every row says nothing.

## Which providers a machine uses

Commands run without `--provider` work on every provider in use
(`src/check/select.ts`). A provider is in use when `config.toml` lists it,
when aicentre runs on it, when a Host block points at it under any name, or,
for Gefion, when tsh has a profile here (`~/.tsh/<proxy>.yaml`). Commands that
work on one provider take the first in use, or the one `-p` names.

## Finding the Host

People name their hosts as they like (`Host gbar.p1`, `Host dtu`, `Host p1`).
aicentre goes by where a block points, its HostName, rather than by its name
(`src/check/aliases.ts`):

1. `alias` in `config.toml`, when set.
2. The default name (`dtu`, `gefion`), when it points at the provider.
3. The first Host in `~/.ssh/config`, following Include, whose HostName is the
   login node and that `ssh -G` confirms. The right host on another port is
   kept as a fallback guess, and `check` says what differs.
4. The default name, when it is configured at all, so `check` can say where it
   points instead.

The scan only proposes names. `ssh -G` decides, so Match blocks,
CanonicalizeHostname and everything else ssh applies still count. Include
globs are expanded in the last path component only, as ssh_config files use
them. Lines may be `Key value` or `Key=value`, with quoted arguments. Wildcard and negated Host patterns are skipped, since nobody can connect
to them by name. `ssh -G` is asked once per alias, and the alias once per
provider.

## The shared login

`src/remote/master.ts` keeps one ssh login per provider, shared by every
aicentre command. DTU asks for a password after the key. So the first command
that needs a provider logs in the way ssh always does, on this terminal,
where the user can type a passphrase or password or trust a host key. ssh then
keeps the login open in the background (`ControlMaster=yes`,
`ControlPersist=30m`, `ServerAliveInterval=30`, `ServerAliveCountMax=4`). Every
command in the next 30 idle minutes, from any terminal, goes through it
without asking again.

- The socket is `cm-<first 12 hex of sha1(alias)>` in the runtime directory.
  Unix socket paths stop at 104 bytes on macOS, and `$TMPDIR` there is long.
  Past 100 bytes the socket moves to `/tmp/aicentre-<uid>`.
- The runtime directory may sit in a shared `/tmp`. It must be a real
  directory (not a symlink), owned by the user, and closed to others, or
  commands stop.
- ssh's messages go to `<socket>.log` with `-E`, not to a pipe. The login
  left running would hold a pipe open, and the reader would never see it
  close.
- With nobody to type (a script, or chat's tools while Pi owns the terminal),
  the login uses `BatchMode=yes` and succeeds only when no prompt is needed.
- Commands through the login use `ControlMaster=no` and `-S <socket>`. A
  streamed command with a terminal (`-tt`) is hung up on, and cleans up, when
  the connection goes. Port forwards are added with `-O forward` and taken
  away with `-O cancel` when the command ends. The login itself stays open.

## Connecting

`src/session/login.ts` is how every session, batch and transfer command
starts. It picks the provider, finds the Host, opens the shared login (saying
"Logging in to <alias>" when it is not open yet), and classifies a failure
with the same classifier as `check` ([06 · Check](06-check.md)). One round
trip then does the setup:

- copies the launcher to `${XDG_CACHE_HOME:-~/.cache}/aicentre/session-<version>.sh`,
  where compute nodes can read it;
- learns the remote user and home directory;
- asks for the installed aicentre version and whether `~/.local/bin` is on
  PATH, for [sync](08-sync-and-transfer.md);
- on DTU, checks that `/dtu/p1/<user>` exists. Without it the command stops
  and asks the user to contact the compute coordinator.

Relative remote paths start in `base`: the project directory on DTU, the home
directory on Gefion. `~` and absolute paths are taken as they are. `data` is
`<base>/.aicentre` and holds tools, caches, uploaded job scripts and logs.

Lines the remote side prints as `@@KEY value` are the answers. Login banners
cannot get in the way of a tag.

## Running on a cluster

`aicentre` also runs on a login node. `src/session/here.ts` decides whether it
is on one, in this order:

1. `AICENTRE_HERE=dtu|gefion|none`.
2. The `provider` file in the data directory, written when a laptop installs
   aicentre on a cluster ([08](08-sync-and-transfer.md)).
3. The machine itself: `bsub` on PATH and `/dtu/p1` for DTU; `sbatch` on PATH
   and a host name containing `dcai` or starting with `slurm-login` for Gefion.

On the cluster, `LocalShell` (`src/remote/shell.ts`) runs the same commands
directly. Commands arrive wrapped in `bash -lc`, because over ssh only a login
shell has the scheduler on PATH. Locally aicentre already runs in the user's
environment, where a login shell adds nothing and can reset PATH, so the
wrapper comes off. `check` reports the provider it runs on as this machine
and runs none of its other steps.
