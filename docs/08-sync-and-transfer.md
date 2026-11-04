# Sync and transfer

## Keeping aicentre on each cluster

`src/session/sync.ts` keeps aicentre on each cluster at the laptop's version.
It is then there when the user logs in, and inside their jobs, which see the
same home directory.

Every connection already asks the login node which aicentre it has
(`@@AICENTRE` from `~/.local/bin/aicentre --version`). When it has none or an
older one, the login node installs this version itself, in the background,
with the same installer as the laptop:

```sh
curl -fsSL https://hpc.aicentre.dk/install | AICENTRE_VERSION=<v> sh
```

So it lands in the cluster's own `~/.local/share/aicentre` and `~/.local/bin`,
and `aicentre uninstall` there removes it. The install runs under `nohup` and
returns at once. It logs to `$XDG_STATE_HOME/aicentre/sync.log` on the
cluster, and on success writes the provider id to
`$XDG_DATA_HOME/aicentre/provider`, which is how
[`here()`](03-providers-and-login.md#running-on-a-cluster) knows where it runs.
A lock directory, `sync.lock`, keeps two laptops or two terminals from
installing at the same time. A lock left by a crash is cleared after 30
minutes.

- A newer copy on the cluster is left alone, with a note that
  `aicentre upgrade` updates the laptop.
- Development builds have no release to install, so they skip sync.
- `sync = false` under the provider in `config.toml`, or `AICENTRE_SYNC=0`,
  turns it off.

The laptop records each cluster it installed on in
`$XDG_STATE_HOME/aicentre/remotes.json`, with the alias, version and time.
`uninstall` reads it to say how to remove those copies. When `~/.local/bin` is
not on PATH in the cluster's login shell, aicentre prints the line to add for
that shell, once per provider, since a copy that is not on PATH is easy to
miss.

## push and pull

`aicentre push <local> [remote]` and `aicentre pull <remote> [local]` copy code
and data to and from a provider (`src/transfer/transfer.ts`). They run rsync
over the [shared login](03-providers-and-login.md#the-shared-login)
(`ssh -S <socket> -o ControlMaster=no`), so a provider that asks for a password
asks once. Like `cp -r`, pushing `datasets/` puts it in `<remote>/datasets`.
Remote paths are absolute, under `~`, or relative to the project directory,
which is also the default.

The flags are `-az --partial`, so a stopped copy picks up where it left off,
plus `--protect-args --info=progress2` on rsync 3.1 or newer, and `--progress`
otherwise. macOS ships an rsync too old for the first two.

On a cluster, push and pull refuse and point at `cp` or `rsync` there. NGC is
air-gapped and takes files only through its staging SFTP. Very large moves
belong on the providers' transfer endpoints (DTU's `transfer.gbar.dtu.dk`,
Gefion's `xfer.dcai.dk`).
