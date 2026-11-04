# Install and release

## Where files go

Everything aicentre writes lives in directories named for it, under the XDG
base directories, on Linux and macOS alike (`src/paths.ts`). Every directory
ends in `aicentre` or `aicentre-<uid>`, which is what lets uninstall remove
them without pointing `rm` at something it does not own. An XDG variable
counts only when it holds an absolute path, as the spec says.

| What | Where |
| --- | --- |
| Program | `$XDG_DATA_HOME/aicentre` (`~/.local/share/aicentre`) |
| Link on PATH | `$XDG_BIN_HOME/aicentre` (`~/.local/bin/aicentre`) |
| Settings | `$XDG_CONFIG_HOME/aicentre` (`config.toml`, `agent/` for chat) |
| State | `$XDG_STATE_HOME/aicentre` (job records, GPU notes, `remotes.json`, chat sessions) |
| Cache | `$XDG_CACHE_HOME/aicentre` |
| Runtime | `$XDG_RUNTIME_DIR/aicentre`, or `$TMPDIR/aicentre-<uid>` (ssh sockets, the upgrade lock) |

The program directory (`src/install/layout.ts`):

```
$XDG_DATA_HOME/aicentre/
  versions/<v>/aicentre     one binary per version; only the current one is kept
  versions/<v>/pi/          the files Pi reads at run time
  current -> versions/<v>   relative, swapped in one rename
  install.json              the version, the platform, and the directories the installer made
~/.local/bin/aicentre -> $XDG_DATA_HOME/aicentre/current/aicentre
```

A new binary is always written to a new file and renamed into place. macOS
caches code signatures per file, and rewriting a signed binary in place can
get it killed on its next start. `current` is swapped in one rename, so a
reader never sees a half state. Its target is relative, so the data directory
can be moved as a whole. The PATH link is only made when nothing else owns
the name. A new binary has to start and report the version it claims before
anything points at it.

`install.json` records the directories outside aicentre's own that the
installer had to create, such as `~/.local/bin` on a fresh Mac. Uninstall
removes them again if they are empty.

Every deletion goes through `safeRemove`, which refuses any path aicentre does
not own: one of its own top-level directories, or something inside its data
or cache directory. Committing a version removes every other version,
including half-finished `.partial` ones.

## install.sh

`pkgs/cli/install.sh` is served at `/install`
([01 · The site](01-the-site.md#endpoints)). It only downloads and verifies a
release, then runs `aicentre self-install` from it, so the layout is written
by one piece of code for a first install, a reinstall or an upgrade. Running
it again reinstalls. It never edits shell startup files. `self-install`
prints the line that puts `~/.local/bin` on PATH for the user's shell (bash,
zsh, fish, tcsh/csh, or `~/.profile`) when it is missing. tcsh reads
`~/.cshrc` when there is no `~/.tcshrc`.

- `AICENTRE_VERSION` installs that version instead of the latest.
- `AICENTRE_DOWNLOAD_BASE` fetches from another URL or a local directory laid
  out like a release. The tests use it.
- The XDG variables are honoured.
- `sh -s -- --allow-root` installs for root, which is refused by default.

It checks the asset against `SHA256SUMS` with `sha256sum` or `shasum`. A shell
under Rosetta reports `x86_64` on an Apple Silicon Mac, so it picks the
native arm64 build. It refuses musl Linux, and NixOS without nix-ld (pointing
at the flake instead). Everything runs from `main`, called on the last line,
so a download cut short runs nothing. The binary is not `exec`ed, because the
EXIT trap has to remove the temporary directory afterwards. stdin under
`curl | sh` is the script itself, so the binary gets `/dev/null`.

## upgrade

`aicentre upgrade [--version v]` fetches a newer release, checks it, switches
to it, and deletes the old one (`src/install/upgrade.ts`). Every step before
the switch works on new files only, in `versions/<v>.partial`, so an upgrade
that fails part way leaves the installed version running and removes its half
made version. The tarball is downloaded into the cache directory, which goes
afterwards, along with `~/.cache` if the upgrade created it. It compares
against the release's `VERSION` file with semver, and does nothing when the
installed version is the latest or newer. One upgrade runs at a time: a lock file in the runtime directory
holds the pid, and a lock left by a process that is gone is taken over.

It refuses a development run, a binary in the Nix store (Nix owns it), and a
copy without `install.json`, which the installer did not set up.

## uninstall

`aicentre uninstall [--purge]` removes the program, its state, cache and
runtime files (`src/install/uninstall.ts`). Settings in `~/.config/aicentre`
belong to the user: `--purge` removes them, a terminal asks (default keep),
and anything else keeps them. Then it removes the directories the installer
created, if they are empty. It reads `remotes.json` before the state goes, and
prints one line per cluster this laptop installed aicentre on, saying how to
remove it there. Those copies belong to the user to remove. A Nix-installed
aicentre is refused, with a pointer to the Nix configuration.

## Release

1. Set `version` in `pkgs/cli/package.json` and merge it.
2. Tag the merge commit and push the tag:

   ```sh
   git tag cli-v0.2.0
   git push origin cli-v0.2.0
   ```

`.github/workflows/release-cli.yaml` checks that the tag matches
`package.json`, runs the typecheck, tests and shellcheck, then builds each
platform on its own runner. Darwin builds are not cross-compiled: bun's ad-hoc
signing of darwin binaries has been unreliable, and macOS kills an unsigned
arm64 binary on start. The macOS builds are signed with `codesign --sign -`. The workflow grants no permissions by default. Only the job that publishes the release can write to the repository.
Each build is smoke-tested: `--version` must match, and `check --json` in an
empty home must exit 0 or 1. The release carries
`aicentre-<os>-<arch>.tar.gz` (the binary and `pi/`), `VERSION` and
`SHA256SUMS`. A tag with a suffix, such as `cli-v0.2.0-rc.1`, becomes a
pre-release. `install.sh` and `upgrade` ignore pre-releases unless asked for
that version.

The asset names are a contract with `install.sh` and the workflow
(`src/platform.ts`): os `linux` or `darwin`, arch `x64` or `arm64`.
`install.sh` fetches from `releases/latest`, so this repository's releases
must stay CLI releases only. The deployed `install.sh` depends on both.
