# Sessions

`aicentre shell|marimo|code [path] [--gpu]` opens an interactive session
(`src/session/interactive.ts`). There are two modes.

- CPU, the default: on the provider's login node, at once. For editing,
  installing and light runs. Servers listen on the login node's `127.0.0.1`
  only. It runs until the user ends it.
- `--gpu`: inside an interactive job with one GPU on `p1i`, DTU's queue for
  interactive work. The job waits for a GPU, then the server listens on the
  compute node and the login node forwards to it. Gefion runs GPU work as
  batch jobs, so `--gpu -p gefion` is refused with a pointer to `submit`.

`--time` (default 60 minutes; `90`, `90m`, `2h`, `1h30m`, `1:30`), `--cpus`
(default 4, at most 64) and `--mem` (memory per CPU, default `8GB`) size a
`--gpu` job and are refused without it. `--server vscode|openvscode` applies
to `code` only. `--no-browser` keeps the browser closed.

Either way one ssh connection carries everything, and when the local process
ends the session ends. Ctrl-C, the job's time limit and the server stopping
all lead to the same cleanup: `bkill` the job, or kill the launcher on a login
node, then drop the port forward. A second Ctrl-C exits at once. In `shell`
the terminal belongs to the remote side, which handles Ctrl-C itself.

On a cluster, only `shell --gpu` works. A CPU shell would open where the user
already is, and marimo and VS Code open in a browser on the laptop.

## The GPU job

`src/session/lsf.ts` builds the bsub line:

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

A login-node session gets a terminal (`ssh -tt`), so the server is hung up on
if the connection drops.

A running GPU session leaves a note in
`$XDG_STATE_HOME/aicentre/gpu/<jobid>.json`. `list jobs` marks the jobs this
laptop is attached to, and `logs` and `cancel` know they run on DTU. Notes
whose session is no longer running here are removed when read.

## The launcher

`src/session/launcher.sh` runs on the cluster. `bun build --compile` embeds it
as text (`import ... with { type: 'text' }`, typed in `src/globals.d.ts`), and
[login](03-providers-and-login.md#connecting) copies it to the login node as
`session-<version>.sh`. Its arguments:

```
bash -l session-<version>.sh <shell|marimo|code> <path> <vscode|openvscode> <data> <base> <bind>
```

- `<path>` is a directory or a notebook: absolute, under `~`, or relative to
  `<base>`, the provider's project directory. It arrives quoted from the
  laptop, so a leading `~` is still literal and the launcher expands it.
- `<data>` holds uv, the editors and their caches.
- `<bind>` is where servers listen: `127.0.0.1` on a shared login node,
  `0.0.0.0` on a compute node, which the login node forwards to.

Lines that start with `@@` are for aicentre on the laptop: `@@PID` first, then
`@@STATUS`, `@@GPU`, `@@ERROR` and `@@LOG` while it starts, then `@@HOST`,
`@@PORT` and `@@PATH` once the server answers. A shell gets plain text,
because a person reads it.

The launcher installs uv into `<data>/bin` when it is missing, for marimo and
for the user's own `uv run`. marimo runs with `uv run --with marimo` when the
directory has a `pyproject.toml`, and with `uvx marimo` otherwise. `code`
fetches the VS Code CLI and runs `serve-web`, or fetches the latest
openvscode-server from GitHub. Every server gets a random token from a
`0600` file, a free port between 20000 and 50000, and a log in
`<data>/logs/<job or login-pid>-<app>.log`. The launcher waits up to 10
minutes for the port to answer. When the server dies first, it sends the
last 8 log lines as `@@LOG`.

On the laptop, aicentre forwards a free local port to `HOST:PORT` (the compute
node for `--gpu`, `127.0.0.1` for a login node) and opens
`http://127.0.0.1:<local><PATH>`. While the session runs, one live line shows
the minutes left on the job, or how to end a login-node session.
