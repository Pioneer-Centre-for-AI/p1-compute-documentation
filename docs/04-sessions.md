# Sessions

`aicentre shell|marimo|jupyter|code [path] [--gpu]` opens an interactive
session (`src/session/interactive.rs`). There are two modes.

- CPU, the default: on the provider's login node, at once. For editing,
  installing and light runs. Servers listen on the login node's `127.0.0.1`
  only. It runs until the user ends it.
- `--gpu`: inside an interactive job with one GPU on `p1i`, DTU's queue for
  interactive work. The job waits for a GPU, then the server listens on the
  compute node's `127.0.0.1`, and aicentre reaches it through the job's gate
  and `sshd -i` (see [on a GPU node](#on-a-gpu-node)). Gefion runs GPU work as
  batch jobs, so `--gpu -p gefion` is refused with a pointer to `sbatch` in
  `aicentre shell`.

`--time` (default 60 minutes; `90`, `90m`, `2h`, `1h30m`, `1:30`), `--cpus`
(default 4) and `--mem` (memory per CPU, default `8GB`) size a `--gpu` job and
are refused without it. The limits come from `h100app`, the LSF application
profile the job is submitted under (`bapp -l h100app` on the login node): 4
tasks, a memory limit of 225 G and a run limit of 720 minutes. `bsub -n 8` is
refused there with "Task request rejected by application profile", so
aicentre stops at 4 CPUs and at 225 GB as CPUs times `--mem`, and says so
before it submits. `--time` stops at 4 hours. That number is aicentre's own
and lower than the profile's 12; longer work is a batch job. A second `--gpu` session from the
same laptop is refused while the first runs, since two would hold every GPU
on the queue. `--another` starts it anyway. `--server vscode|openvscode` applies
to `code` only. `--no-browser` keeps the browser closed. `code` also takes
`--desktop` and `--browser`, described in [desktop editors](#desktop-editors).
`--server` and `--no-browser` mean the browser, so either one with
`--desktop` is a usage error.

Either way one ssh connection carries everything, and when the local process
ends the session ends. Ctrl-C, the job's time limit and the server stopping
all lead to the same cleanup: `bkill` the job, or kill the launcher on a login
node, then drop the port forward. A second Ctrl-C exits at once. In `shell`
the terminal belongs to the remote side, which handles Ctrl-C itself.

A session that ends another way, with the connection lost or the laptop
asleep, also runs `bkill` for its job, since the job may not have noticed.
aicentre says a job ended only when `bkill` accepted it or LSF answers that
it already finished. When neither can be had, it says the job may still hold
a GPU and how to end it, and exits 1. ssh exits 255 when its connection
fails, so that code, or a signal, makes the last line say the connection was
lost; any other code says the server stopped. A GPU shell sets bash's `TMOUT`, so 10
minutes at an idle prompt end it and its job; the other GPU sessions have the
launcher's [idle watch](#on-a-gpu-node).

On a cluster, only `shell --gpu` works. A CPU shell would open where the user
already is, and marimo, JupyterLab and VS Code open on the laptop.

## The GPU job

`src/session/lsf.rs` builds the bsub line:

```sh
ESUB_BYPASS=1 ESUB_QUIET=1 bsub -I -q p1i -app h100app -J aicentre-<app> \
  -env none -W <h:mm> -n <cpus> -R 'rusage[mem=<mem>]' -R 'span[hosts=1]' \
  -gpu num=1:mode=shared bash -l <launcher> <app> <path> <server> <data> <base> 0.0.0.0
```

DTU's submission check rejects `-W` on `p1i` unless it is bypassed, and
without `-W` a session ends after 15 minutes, hence `ESUB_BYPASS`.
`-env none` keeps the login node's environment out of the job, and `bash -l`
builds a fresh one. `rusage[mem=...]` is per CPU, as LSF counts it on DTU. A
GPU shell uses `-Is`. marimo and VS Code use `-I`, and the output of `bsub -I`
is read line by line while the job waits, starts and runs.

LSF does not hand the job command to the node as separate words. It joins
them into one line of a job script, with quoting of its own that leaves `;`,
backticks, `|` and `>` bare, and the node's shell reads that line again. On
DTU a path such as `a;id` ran `id` in the job. So `<path>`, `<data>` and
`<base>` go as `hex:<hex of the bytes>` whenever they hold anything but
letters, digits and `_@%+=:,./-`, and the launcher decodes them. The login
node's launcher gets its arguments from one shell, quoted once, and needs
none of this.

A login-node session gets a terminal (`ssh -tt`), so the server is hung up on
if the connection drops. The login node is shared, so the launcher keeps a
session there small:

- The server runs under `nice -n 10`, in its own process group (`setsid`),
  and the cleanup kills the group, so a server's helpers go with it.
- Every 30 seconds the launcher prints `@@BEAT`. A connection that died
  without a hangup, as when a laptop sleeps, fails that write once TCP gives
  up, and the session ends then instead of when sshd's keepalive notices.
- A session ends after 12 hours with `@@ERROR`. JupyterLab also stops kernels
  idle for an hour, and itself after an hour with none.

None of this limits memory or cores. The node's own limits do that, and real
compute belongs in `--gpu`.

A running GPU session leaves a note in
`$XDG_STATE_HOME/aicentre/gpu/<jobid>.json`. `list jobs` marks the jobs this
laptop is attached to. Notes whose session is no longer running here are
removed when read.

## The launcher

`src/session/launcher.sh` runs on the cluster. The binary carries it as text
(`include_str!` in `src/session/login.rs`), and
[login](03-providers-and-login.md#connecting) copies it to the login node as
`session-<version>.sh`. Its arguments:

```
bash -l session-<version>.sh <shell|marimo|jupyter|code|sshd> <path> <vscode|openvscode> <data> <base> <bind>
```

- `<path>` is a directory or a notebook: absolute, under `~`, or relative to
  `<base>`, the provider's project directory. With no path, aicentre sends
  the remote home, so a session opens where people keep their code; relative
  paths stay project-relative because `push` and `gallery get` put files
  there. The launcher sends `@@DIR`, the folder it opened, and the ready
  block shows it under the URL. A path arrives quoted from the laptop, so a
  leading `~` is still literal and the launcher expands it.
- `<data>` holds uv, the editors and their caches.
- `<bind>` is where the gate listens in a job: `0.0.0.0`, so the login node
  can reach it. Servers always listen on `127.0.0.1`. A login-node session
  passes `127.0.0.1` and starts no gate.

Lines that start with `@@` are for aicentre on the laptop: `@@PID` first, then
`@@STATUS`, `@@GPU`, `@@ERROR` and `@@LOG` while it starts, then `@@HOST`,
`@@PORT` and `@@PATH` once the server answers. `sshd` adds `@@HOSTKEY` and
`@@DIR` before those, and its `@@PATH` is empty. A shell gets plain text,
because a person reads it.

The laptop treats those lines as coming from a cluster account that may be
compromised. Control characters are removed from every line before it is
read or shown, so no escape sequence reaches the terminal. `@@HOST` must be
a host name, `@@PATH` a URL path with at most one `name=token` query,
`@@DIR` absolute, and `@@PID`, `@@PORT` and `@@GATE` numbers, as `@@SECRET`
and `@@HOSTKEY` already had to fit their forms. Otherwise the session fails
and ends its job, before a browser, an editor or a port forward sees the
value.

The launcher installs uv into `<data>/bin` when it is missing, for marimo,
JupyterLab and the user's own `uv run`. marimo runs with
`uv run --with marimo` when the directory has a `pyproject.toml`, and with
`uvx marimo` otherwise. JupyterLab works the same way, with
`uv run --with jupyterlab jupyter-lab` or `uvx --from jupyterlab jupyter-lab`.
jupyterlab depends on ipykernel, so the kernel is in the same environment, and
with a `pyproject.toml` it sees the project's packages. `code` fetches the VS
Code CLI and runs `serve-web`, or fetches openvscode-server from GitHub.
`sshd` is for [desktop VS Code on a GPU node](#on-a-gpu-node).

The three programs the launcher downloads and runs are pinned in
`launcher.sh`: uv by version, openvscode-server by release, the VS Code CLI
by commit, each with the sha256 of the one file it expects. `fetch` downloads
over https only, compares the hash, and on a mismatch deletes the file and
ends the session with both the source and the hash it got. Before, uv came
from `curl https://astral.sh/uv/install.sh | sh` and the editors from
whatever "latest" pointed at, so a session ran whatever those sites served
that day. A new version reaches users with an aicentre release that changes
the pin. The hashes come from the publishers: GitHub's release API lists a
`digest` per asset, and
`https://update.code.visualstudio.com/api/update/cli-alpine-x64/stable/latest`
gives `version` (the commit) and `sha256hash`. The editors are x86_64 only,
as before, and now say so on another machine. uv has both pins.
`AICENTRE_FETCH_FROM` points the downloads at another place for the tests; it
does not change the expected hash.

Two downloads have no hash in the launcher. `serve-web` fetches the VS Code
server itself, from Microsoft; `--commit-id` holds it to the CLI's commit.
marimo and JupyterLab come
from PyPI through uv at their latest version, with the project's own lock
file when there is one; uv checks each file against the hash the index
gives.

Every server gets a random token, a free port between 20000 and 50000, and a
log in `<data>/logs/<job or login-pid>-<app>.log`. marimo and VS Code read the
token from a `0600` file. JupyterLab has no token-file option, so it gets
`JUPYTER_TOKEN` in its environment, which keeps the token out of `ps`. With
`--no-browser`, JupyterLab does not open a notebook given on its command line,
so the launcher puts the notebook in the URL instead, as
`/lab/tree/<percent-encoded name>?token=...`. JupyterLab writes its URL,
token included, into its log. So the launcher makes `<data>/logs` `0700` and
creates each log `0600` before a server writes to it. Each start deletes
logs, and launchers of other versions, that are older than 14 days. On DTU
`/dtu/p1/<user>` is `0700` already, so nothing was readable by others
before. The rule is for project directories that are not.

The port is picked before the server starts, and the node is shared, so
another user could listen on it first and be handed the URL with the token.
The launcher therefore reports `@@PORT` and `@@PATH` only when the listener
is this user's own process: `ss -Hltnp` names a pid only for one's own
sockets, and `lsof -u` does the same where there is no `ss`. With neither
tool, or with a foreign listener, the session fails and nothing is printed.
JupyterLab runs with `--ServerApp.port_retries=0`, so it cannot move to a
neighbouring port and still accept the token.

The launcher waits up to 10 minutes for the port to answer. When the server
dies first, it sends the last 8 log lines as `@@LOG`.

On the laptop, aicentre opens `http://127.0.0.1:<local><PATH>`. For a
login-node session, `<local>` is forwarded through the shared login to the
login node's `127.0.0.1:PORT`. In a job, the launcher starts the gate in
forward mode before the server (`@@SECRET`, `@@GATE`). aicentre forwards a
port through the shared login to the gate, and serves `<local>` itself with a
small relay (`secret_proxy` in `proxy.rs`) that sends the secret first on each
connection. The gate then connects to the server on the node's `127.0.0.1`.

That needs no second login. An earlier version tunnelled browser sessions
through the job's `sshd -i`, which encrypted the last hop too, but it needed
key-based login into the job: people who reach DTU with a password, or whose
key sits in an agent named in their ssh config, lost browser GPU sessions. So
the hop from the login node to the GPU node is plain HTTP inside the cluster,
as it always was. Only someone with root on the cluster could read it. What
changed is that the server no longer has a port on the node's network.

A browser left closed for 10 minutes ends the job, the same as a desktop
editor: every connection through the gate is a child of the gate, and a
browser tab holds a websocket while it is open. While the session runs, one
live line shows the minutes left on the job, or how to end a login-node
session.

## Desktop editors

`aicentre code` opens the user's own desktop editor when it can, and VS Code
in the browser otherwise (`src/session/editor.rs`).

The contract with editors is an ssh host and a path. Every editor that works
on remote folders, and every ssh tool, understands that much: the VS Code
family, Zed, JetBrains Gateway, Emacs TRAMP, `ssh`, `scp` and `rsync`. So
aicentre's job is to make the host exist, and the editor-specific part is kept
to launching. On a login node the host is the user's own alias. In a GPU job it
is `aicentre-gpu`, from aicentre's own file (below). The ready output always
prints `<host>:<path>`, for any tool aicentre does not launch.

Launching is a command template, not code per editor. `[desktop] open` in
`config.toml` names any editor, with `{host}`, `{path}` and `{urlpath}`
(the path percent-encoded per segment) filled in per word. Without it,
aicentre knows two templates, as data:

| Commands | Template | Ready when |
| --- | --- | --- |
| `code`, `codium`, `cursor`, `windsurf` | `--folder-uri vscode-remote://ssh-remote+{host}{urlpath}` | `--list-extensions` lists an id with `remote-ssh` or `remote-openssh` in it |
| `zed`, `zeditor` | `ssh://{host}{urlpath}` | always; remote editing is built in |

The commands are tried in the order of the table. If the first word of
`$VISUAL` or `$EDITOR`, checked in that order, is one of them by name or path,
that command goes first: `EDITOR="codium --wait"` puts VSCodium ahead of VS
Code, and `EDITOR=vim` changes nothing. The first command found on the PATH
that is ready wins. Only VS Code and VSCodium have been tried against P1. The
VS Code forks share the CLI and the `ssh-remote+` authority, and Zed resolves
hosts through `~/.ssh/config`
([Zed's docs](https://zed.dev/docs/remote-development)). Zed has an open
issue where hosts behind a ProxyCommand time out
([#30702](https://github.com/zed-industries/zed/issues/30702)), which would
hit `aicentre-gpu` and Teleport's Gefion Host. On macOS the editor's shell
command must be on the PATH, or `open` names the CLI inside the app bundle.

With `--gpu`, the desktop editor is used only when `--desktop` asks for it.
It needs an sshd of the user's own inside the job
([below](#on-a-gpu-node)), DTU refuses ssh from its login node to its compute
nodes, and no DTU page says a user may run one. Until DTU says so, plain
`code --gpu` opens the browser and says why, so nobody runs the sshd without
having chosen to.

The desktop editor is picked when there is a display (always on macOS,
`DISPLAY` or `WAYLAND_DISPLAY` on Linux, the same test `open_url` uses), an
editor is ready, and, with `--gpu`, `ssh -G aicentre-gpu` has a ProxyCommand
that runs `aicentre proxy`, which means `~/.ssh/config` includes aicentre's
file. The last is checked after login, when aicentre has written that file.
When an editor is installed but something is missing, one dim line says what,
and the browser opens. With no editor at all, nothing is said. `--desktop`
turns the same reasons into a failure. `--browser` skips the detection.

### On a login node

aicentre logs in as for any session, which gives the provider, the alias, the
project directory and the sync. It checks that the path exists. A file opens
its folder, as in the browser. Then it runs the editor's template with the alias and the folder, and
exits. The editor makes its own connection with the user's `~/.ssh/config`,
the same Host aicentre found, and the session lasts until the editor closes it.
Remote editors install their server in the remote home. On DTU that home holds
30 GB, and VS Code keeps the last 5 server versions there. The settings in the
gallery's [rem-lgn](../pkgs/gallery/rem-lgn/settings.json) example move it to
project storage with `remote.SSH.serverInstallPath`. That is a VS Code setting,
so aicentre cannot set it without editing the user's VS Code settings, and
does not. `VSCODE_AGENT_FOLDER` is not a way around this: VSCodium's
Remote-SSH ignores it.

### On a GPU node

Remote-SSH needs an SSH server on the GPU node. The job runs its own, as the
user. The launcher's `sshd` app:

- finds `sshd` (`command -v sshd`, else `/usr/sbin/sshd`). It needs the
  absolute path to re-exec.
- makes a `mktemp -d` directory (`0700`, on the node's own disk) with a fresh
  ed25519 host key and an `sshd_config`. The directory goes when the job ends.
- reads keys from the user's own `~/.ssh/authorized_keys`, the same file the
  login node uses, with `StrictModes` left on.
- writes `SetEnv CUDA_VISIBLE_DEVICES=... LSB_JOBID=...`. sshd builds a fresh
  environment for each session, so without this, terminals in VS Code would
  not see the job's GPU. `SetEnv` in `sshd_config` needs OpenSSH 8.7; the DTU
  nodes have 9.9.
- makes a 32-byte random secret, as 64 hex characters in a `0600` file.
- starts a gate on the free port instead of sshd: a short standard-library
  Python script (`python3`, else `/usr/libexec/platform-python`), written into
  the private directory. sshd itself never listens.
- sends `@@HOSTKEY`, `@@SECRET` and `@@DIR`, and the usual wait for the port
  gives `@@HOST`, `@@PORT` and an empty `@@PATH`.

The gate first drops a connection whose address is not the submitting login
node's. Tests set the list with `AICENTRE_GATE_ALLOW`, and point sshd at a
test key with `AICENTRE_AUTHORIZED_KEYS`, which also turns `StrictModes` off
for that file. `bsub -env none` keeps both out of real jobs. Then it forks
once per connection, reads up to 64 bytes within 5 seconds in all, and
compares them with the secret using
`hmac.compare_digest`. On a mismatch the connection closes without a byte sent
back. On a match, in `sshd` mode the socket becomes stdin and stdout of
`sshd -i -e -f <config>`, inetd mode, for that one connection. In `forward`
mode the child connects to the server on `127.0.0.1` and copies bytes both
ways. A child that passed the secret says so to the gate through a pipe. The
gate holds at most 16 connections, of which at most 4 may still be waiting to
send the secret, so other users of the login node cannot fill it. It wakes
at least once a second, reaps finished children, and writes the number of
connections that passed the secret to a file. The gate and its sshd log to
`<data>/logs/<job>-gate.log`, apart from the server's own log, and the launcher
stops with the end of that log if the gate dies. The sshd config is checked
with `sshd -t` before the gate starts, and `SetEnv` lists only the variables
the job has, so a job without `CUDA_VISIBLE_DEVICES` does not get an empty one
that hides the GPU. `MaxAuthTries` stays at its default: the gate already
keeps strangers out, and a lower value fails people with several keys in their
agent. So someone else on the
cluster network reaches a 64-byte comparison, not OpenSSH's handshake. On DTU
in November 2026 a connection from login9 without the secret got no answer,
and nothing reached sshd's log.

Every process the gate and sshd start is a child of the job, so it is in the
job's cgroup (`/lsf/dcc/job.<id>...` on DTU), and `bkill` or the time limit
ends it.

A desktop editor can be closed while the job keeps its GPU. So after the
server is ready, the launcher reads the gate's count of connections that
passed the secret, every 30 seconds. A connection that never sent the secret
does not count. After 10 minutes with none, it sends `@@ERROR`
with the reason and exits, and the job ends. That covers both an editor that
never connected and one closed for good, and a laptop asleep for more than 10
minutes loses its job. The sleep is `sleep & wait`, so Ctrl-C does not wait
for it. `AICENTRE_IDLE_SECONDS` changes the limit for tests; `bsub -env none`
keeps it out of real jobs.

On the laptop there is no port forward. aicentre checks that the host key is
ed25519 and the secret is 64 hex characters, writes the session note (`0600`)
with the alias, node, gate port, host key, secret and host name, rewrites
`$XDG_STATE_HOME/aicentre/known_hosts` from the live notes as
`aicentre-gpu-<job>,aicentre-gpu <key>`, and opens the editor on that host.
The host name is `aicentre-gpu` unless another live session already has it;
then it is `aicentre-gpu-<job>`. One name for the usual single session means
Remote-SSH asks for the platform once and keeps one recent entry. A second
session gets its own name, so a reconnecting window never lands in the other
job. The ready block, the clock and Ctrl-C work as for a browser session. When
the session ends, the note goes and `known_hosts` is rewritten without its
key.

Remote-SSH finds hosts only through ssh config, so `aicentre-gpu` needs an
entry there. aicentre never edits `~/.ssh/config`. It owns
`$XDG_STATE_HOME/aicentre/ssh_config` and rewrites it on every
`aicentre code --gpu` and every `aicentre check` that has an editor to check
for (`write_ssh_config` in `proxy.rs`, the block from `gpu_host_block` in
`clusters.rs`):

```
# Written by aicentre, which keeps it up to date.
Host aicentre-gpu aicentre-gpu-*
  User <dtu-username>
  ProxyCommand ~/.local/bin/aicentre proxy %n
  UserKnownHostsFile ~/.local/state/aicentre/known_hosts
  StrictHostKeyChecking yes
  ServerAliveInterval 30
```

The user adds one line, once, at the top of `~/.ssh/config`:
`Include ~/.local/state/aicentre/ssh_config`. With Home Manager that is
`programs.ssh.includes`, which puts it there. It has to come before any
`Host *` block: ssh keeps the first value it reads for each option, so a
`Host *` above it could set `User` or `UserKnownHostsFile` first. An Include
of a file that does not exist yet is not an error. This is the model of
`gh codespace ssh --config`: the tool owns a file, and the user includes it.
Coder and gcloud edit `~/.ssh/config` themselves, between markers.

The User is the remote `$USER` from the login, or in `check` the DTU Host's
User. IdentityFile lines come from the DTU Host, leaving out the default
names ssh tries anyway.

`aicentre proxy <host>` (`src/session/proxy.rs`) reads the live notes. It
takes the desktop session whose host name it was given (`aicentre-gpu`), or
job `<id>` for `aicentre-gpu-<id>`. Browser sessions keep no host key or
secret in their note, so it never picks one. It runs
`ssh -S <shared socket> -o ControlMaster=no -W <node>:<port> <alias>`. It
writes the secret first, then passes its own stdin through. That is a stdio
forward through the shared login the session already holds, so it asks for
nothing. Its stdout is the SSH stream, so it prints only to stderr. The secret
never appears in a command line or in ssh config.
The ProxyCommand names `~/.local/bin/aicentre` when that link exists. ssh
started by a desktop app may not have `~/.local/bin` on its PATH. Otherwise it
names the running binary, unless that is in the Nix store, where the path
changes with every upgrade and plain `aicentre` from the PATH is used. For Remote-SSH, the
`aicentre-gpu` entries in rem-lgn's `settings.json` set the platform and keep
the server in project storage.

### Shared nodes

`p1i` nodes and the login node are shared with other users. The design holds
against them as follows.

- Only keys in the user's `authorized_keys` log in. A sshd running as the user
  cannot log in as anyone else, and `PasswordAuthentication` is off.
- The host key is new for each session. It reaches the laptop over the
  authenticated login and is pinned with `StrictHostKeyChecking yes`, so
  nobody else can answer as the node.
- For a desktop editor, SSH encrypts the whole path from the laptop to the
  node. A browser session is encrypted as far as the login node, and plain
  HTTP between the login node and the node, behind the gate and the server's
  token. Servers on a GPU node listen on its `127.0.0.1` only. Other users on
  that node can still reach `127.0.0.1`, as on a login node, so the server's
  own token still guards it.
- The port is reachable from the cluster network, and a non-root sshd has no
  privilege separation, so sshd does not listen: the gate above answers the
  port, and only a connection with the session's secret reaches sshd. Then
  `AllowUsers <user>@<ip>` takes the addresses of
  `$LSB_SUB_HOST`, the login node that submitted the job, which is where
  `ssh -W` connects from (`10.66.10.43` for `hpclogin9` on DTU). Loopback
  addresses are left out: the login node resolves its own name to
  `127.0.0.1` as well, and loopback on the GPU node would mean the other users
  on it. If no other address is left, the launcher stops with `@@ERROR`
  rather than take connections from anywhere, and `--browser` still works.
  `LoginGraceTime 20` limits what a session can try, and the gate's limits
  of 16 connections and 4 waiting ones what a flood can hold.
  `AllowTcpForwarding local` is enough for VS Code's `-D`, and agent
  forwarding, X11 and tunnels are off.
- ssh into the node itself, where VS Code would use DTU's own sshd, is not a
  way around this: from login9 it was refused ("Permission denied ...
  hostbased"), and sessions started that way would sit outside the job.

### Why not blaunch or battach

`blaunch` starts tasks inside a job, but only from inside its allocation. It
checks `LSB_JOBID` and `LSB_MCPU_HOSTS` with the job's RES, and from a login
node it stops with "Cannot run outside LSF". The launcher already runs inside
the job on its one host, so blaunch would add nothing there.

`battach` attaches to a running job from outside and puts what it starts in
the job's cgroup, with nothing listening. `battach -L <wrapper> <job>`
running `sshd -i` would have been the cleaner path. On DTU (LSF 10.1.0.16,
November 2026) it answers "Cannot attach jobs that are not running" for a
running `bsub -I` job, and `p1i` refuses batch jobs ("Queue only accepts
interactive jobs"). battach also prints its errors on stdout, which would
land in the SSH stream.
