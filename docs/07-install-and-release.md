# Install and release

## Where files go

Everything aicentre writes lives in directories named for it, under the XDG
base directories, on Linux and macOS alike (`src/paths.rs`). Every directory
ends in `aicentre` or `aicentre-<uid>`, which is what lets uninstall remove
them without pointing `rm` at something it does not own. An XDG variable
counts only when it holds an absolute path, as the spec says.

| What | Where |
| --- | --- |
| Program | `$XDG_DATA_HOME/aicentre` (`~/.local/share/aicentre`) |
| Link on PATH | `$XDG_BIN_HOME/aicentre` (`~/.local/bin/aicentre`) |
| Settings | `$XDG_CONFIG_HOME/aicentre` (`config.toml`) |
| State | `$XDG_STATE_HOME/aicentre` (GPU notes, `remotes.json`, `ssh_config`, `known_hosts`) |
| Cache | `$XDG_CACHE_HOME/aicentre` |
| Runtime | `$XDG_RUNTIME_DIR/aicentre`, or `$TMPDIR/aicentre-<uid>` (ssh sockets, the upgrade lock) |

State is private. Session notes hold the session's URL with its token.
`state_dir` in `src/paths.rs` creates the
directory `0700` and tightens one that a release before 0.2 left `0755`,
and the files in it are `0600`. aicentre also runs on the clusters, where it
cannot count on the home directory being private. The runtime directory is
`0700` for the same reason. On a cluster, the launcher makes its `<data>`
directory (`.aicentre` in the project directory) `0700`.

The program directory (`src/install/layout.rs`):

```
$XDG_DATA_HOME/aicentre/
  versions/<v>/aicentre     one binary per version; only the current one is kept
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

Every deletion goes through `safe_remove`, which refuses any path aicentre does
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
`~/.cshrc` when there is no `~/.tcshrc`. It also prints the command that
adds it to the current shell, because `curl | sh` runs in a child process and
cannot change the caller's PATH. fish gets no second line: `fish_add_path`
applies at once.

- `AICENTRE_VERSION` installs that version instead of the latest.
- `AICENTRE_DOWNLOAD_BASE` fetches from another URL or a local directory laid
  out like a release. The tests use it.
- The XDG variables are honoured.
- `sh -s -- --allow-root` installs for root, which is refused by default.

It checks `SHA256SUMS` against `SHA256SUMS.sig` with `ssh-keygen -Y verify`
and aicentre's release key ([below](#the-release-signature)), then the asset
against `SHA256SUMS` with `sha256sum` or `shasum`. A release without the
signature is not installed. A shell
under Rosetta reports `x86_64` on an Apple Silicon Mac, so it picks the
native arm64 build. The Linux builds from 0.2 on link musl statically and run
on any distribution. For a 0.1 release, which needed glibc, it refuses musl
Linux and NixOS without nix-ld, and points at the flake. Everything runs from `main`, called on the last line,
so a download cut short runs nothing. The binary is not `exec`ed, because the
EXIT trap has to remove the temporary directory afterwards. stdin under
`curl | sh` is the script itself, so the binary gets `/dev/null`.

## upgrade

`aicentre upgrade [--version v]` fetches a newer release, checks it, switches
to it, and deletes the old one (`src/install/upgrade.rs`). Every step before
the switch works on new files only, in `versions/<v>.partial`, so an upgrade
that fails part way leaves the installed version running and removes its half
made version. The tarball is downloaded into the cache directory. Afterwards
the tarball and the stage directory go, and so do the cache directories the
upgrade created, if they are empty. Other files in the cache stay. It compares
against the release's `VERSION` file with semver, and does nothing when the
installed version is the latest or newer. One upgrade runs at a time: a lock file in the runtime directory
holds the pid, and a lock left by a process that is gone is taken over.

It refuses a development run, a binary in the Nix store (Nix owns it), and a
copy without `install.json`, which the installer did not set up.

## uninstall

`aicentre uninstall [--purge]` removes the program, its state, cache and
runtime files (`src/install/uninstall.rs`). Settings in `~/.config/aicentre`
belong to the user: `--purge` removes them, a terminal asks (default keep),
and anything else keeps them. Then it removes the directories the installer
created, if they are empty. It reads `remotes.json` before the state goes, and
prints one line per cluster this laptop installed aicentre on, saying how to
remove it there. Those copies belong to the user to remove. A Nix-installed
aicentre is refused, with a pointer to the Nix configuration.

## Release

1. Set `version` in `pkgs/cli/Cargo.toml` and merge it.
2. Tag the merge commit and push the tag:

   ```sh
   git tag cli-v0.2.0
   git push origin cli-v0.2.0
   ```

`.github/workflows/release-cli.yaml` checks that the tag matches
`Cargo.toml`, runs the tests and shellcheck, then builds each platform on its
own runner. The Linux targets are `x86_64-unknown-linux-musl` and
`aarch64-unknown-linux-musl`, built with `musl-gcc` for ring's C code, and a
binary that is not statically linked fails the job. macOS kills an unsigned
arm64 binary on start, so the macOS builds are signed with
`codesign --sign -` and verified. The workflow grants no permissions by
default. Only the job that publishes the release can write to the repository.
Each build is smoke-tested: `--version` must match, and `check --json` in an
empty home must exit 0 or 1. The release carries
`aicentre-<os>-<arch>.tar.gz` (the binary alone), `VERSION`, `SHA256SUMS`
and `SHA256SUMS.sig`. A tag with a suffix, such as `cli-v0.2.0-rc.1`, becomes a
pre-release. `install.sh` and `upgrade` ignore pre-releases unless asked for
that version.

After publishing, the `install-check` job runs `scripts/install-check.sh` on
the same four runners against the real release. It installs the previous
stable release with the tag's `install.sh`, upgrades to the new version,
upgrades again (which must report it as installed), and runs
`uninstall --purge`. Then it installs the new version fresh, runs `check`, and
uninstalls without `--purge`, which must keep `config.toml`. After each
uninstall, aicentre's binary link and data, state and cache directories must
be gone. The script runs as any user without aicentre:

```sh
pkgs/cli/scripts/install-check.sh 0.2.0 0.1.1
```

### The release signature

A checksum file that sits next to the download proves the download arrived
whole. It says nothing about who published it: whoever can replace a release
asset can replace `SHA256SUMS` too. So the publish job signs `SHA256SUMS`
with an Ed25519 SSH key, and `install.sh`, `upgrade` and the install a
cluster does for itself ([08](08-sync-and-transfer.md)) refuse a release
whose `SHA256SUMS.sig` is missing or was not made with that key.

- The public key is `pkgs/cli/release.pub`, one line in OpenSSH's
  allowed-signers format. The binary embeds it. `install.sh` is served on its
  own, so it carries a copy, and a test fails when the two differ.
- The private key is the repository secret `AICENTRE_RELEASE_KEY`. The
  publish job fails when it is not set, and verifies its own signature
  against `release.pub` before it creates the release.
- The format is OpenSSH's file signature (`ssh-keygen -Y sign`, namespace
  `aicentre-release`), because every machine that can use aicentre has
  OpenSSH. `install.sh` verifies with `ssh-keygen -Y verify`, which needs
  OpenSSH 8.1. The binary verifies the same file itself
  (`src/install/signature.rs`, with ring), so `upgrade` does not depend on
  the ssh-keygen it finds.
- `AICENTRE_RELEASE_SIGNERS` names another allowed-signers line, and counts
  only together with `AICENTRE_DOWNLOAD_BASE`. The tests sign their fake
  releases with a key they make.
- Releases from before the signature have none. The workflow
  `sign-cli-releases.yaml` signs every `cli-v*` release that has
  `SHA256SUMS` and no signature. It runs when `release.pub`, the workflow or
  `scripts/sign-release.sh` changes on `main`, so the push that brings the
  signature check in also signs what was released before it. Run by hand
  with a tag, it signs that release again. It calls
  `scripts/sign-release.sh <private key> [<tag>...]`, which also runs on a
  laptop.
- The website serves `install.sh` from `main`, so an installer that asks for
  a signature must not go live while the latest release has none. The
  deploy workflow therefore runs the built `site/install` in an empty home
  against the real latest release, and does not deploy when that fails. It
  tries for three minutes, since the signing workflow starts on the same
  push. This also stops any other change that breaks the installer.
- Setting the secret is the one step done by hand, once:
  `gh secret set AICENTRE_RELEASE_KEY < <private key>`. No workflow can
  write a secret with the token it is given.
- The publish job also records a build attestation for each archive
  (`actions/attest-build-provenance`). It ties the archive to the workflow
  run and commit that built it, with no key to keep, and
  `gh attestation verify aicentre-linux-x64.tar.gz --repo <repo>` checks it.
  The installer does not use it: verifying needs `gh` or a Sigstore client,
  and a login node has neither.
- To change the key, replace the line in `release.pub` and in `install.sh`,
  set the secret, and release. Installed versions trust the old key, so they
  upgrade through a release signed with it, or by running `install.sh` again.

### Release candidates

Test a release on real downloads before users get it:

1. Set `version` to `0.2.0-rc.1`, commit, and push `cli-v0.2.0-rc.1`. The
   pre-release does not become `releases/latest`, so `install.sh` and
   `upgrade` do not offer it.
2. If `install-check` fails, fix it and release `rc.2`.
3. When it passes on all four runners, set `version` to `0.2.0` and push
   `cli-v0.2.0`. `install-check` runs again on the real latest release.

The tag must equal `Cargo.toml`, so the binary's `--version`, the release's
`VERSION` file and the tag agree.

A 0.1 install reaches 0.2 through its own TypeScript `upgrade`, so the
tarball keeps `aicentre` at its root, and the Rust binary reads the same
`install.json`. 0.1's `pi/` directory goes with the old version directory.
`install-check` tests this path on every release.

The asset names are a contract with `install.sh` and the workflow
(`src/platform.rs`): os `linux` or `darwin`, arch `x64` or `arm64`.
`install.sh` fetches from `releases/latest`, so this repository's releases
must stay CLI releases only. The deployed `install.sh` depends on both.
