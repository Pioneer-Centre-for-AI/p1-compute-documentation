# Testing

```sh
cd pkgs/cli
cargo test                   # unit and scenario tests
AICENTRE_E2E=1 cargo test    # also the end-to-end install run and a real sshd
cargo clippy --all-targets -- -D warnings
cargo fmt --check
shellcheck -s sh install.sh
shellcheck -s bash src/session/launcher.sh scripts/install-check.sh
```

The tests cover core behaviour only. They are kept few on purpose. Unit tests
for pure functions (paths, parsing, classification, the decisions commands
make) sit next to the code in `#[cfg(test)]` modules. The scenario tests are
in `tests/`:

| File | Covers |
| --- | --- |
| `tests/help.rs` | Every help page fits 80 columns and carries no colour when piped |
| `tests/check.rs` | `aicentre check` end to end, against fake ssh, ssh-keygen, ssh-add and tsh |
| `tests/sessions.rs` | `shell`, `marimo`, `jupyter` and `code`, on the login node and with `--gpu`, in the browser and in a desktop editor |
| `tests/proxy.rs` | `aicentre proxy` and the session it reaches |
| `tests/jobs.rs` | `list jobs`, `list gpus`, `gallery get` and `push`, against a fake ssh that plays the login node, LSF and Slurm |
| `tests/mcp.rs` | `aicentre mcp`: the protocol, cancellation, the answer cache and `p1_docs` |
| `tests/launcher.rs` | `launcher.sh` itself, run with bash against stand-in `uvx` and `sshd` that only open the port: JupyterLab's token and URL, private logs, the gate (a wrong secret gets nothing, the right one reaches sshd), the sshd config (checked with the real `sshd -t` when one is installed), the idle release, and cleanup. With `AICENTRE_E2E=1` on Linux, where sshd is installed, one test runs a real `sshd -i` behind the gate and logs in with a real `ssh`; a wrong secret must fail |
| `tests/install.rs` | `upgrade` and `uninstall` against a fake release, and with `AICENTRE_E2E=1` the real thing |

The scenario tests call `aicentre::run(argv, &ctx)` in the test process, with
a `Ctx` built around the sandbox. Its output is captured and 100 columns wide,
its version is `0.1.0` and its executable `/nonexistent/aicentre`. Sessions
take their browser, editor and free ports from a `SessionDeps` the test fills
in, and a cancellation token stands in for Ctrl-C.

## No goldens

During the port `tests/golden/` held what the TypeScript build printed, and the
Rust build had to match it byte for byte. That proved the port. The
TypeScript build is gone, so the goldens went too, and with them the code
that copied JavaScript's behaviour: its sort, its number parsing and its
`String()`. Jobs are sorted by state, then job number, then id, so the tasks
of an array job sit next to their job.

## The sandbox

The tests never touch the developer's own `~/.ssh`. OpenSSH takes the home
directory from the password database rather than `$HOME`, so a real ssh would
read the real `~/.ssh` whatever the test sets. `tests/common/mod.rs` makes a
throwaway HOME with fake `ssh`, `ssh-keygen`, `ssh-add` and `tsh`
(`tests/common/fakes.rs`) first on a PATH that holds only the few real tools
the code needs (`sh`, `bash`, `tar`, `gzip`, `cat`, `env`, `sleep`, `touch`,
`rm`, `mkdir`, `head`). The checks only ever meet the fakes.

The fakes are shell scripts driven by files in `$FAKE/answers/<name>`, which a
test writes with `sb.answer()`. `ssh -G <alias>` prints
`answers/ssh-G-<alias>`, or the default for an unconfigured host. The fake ssh
plays the shared login too: `ControlMaster=yes` touches `$FAKE/master.up`,
which stands for an open login, and `-O check` tests for it. For a login probe,
the alias comes right before the remote command, and the answers are
`probe-<alias>.stdout`, `.stderr` and `.code`. The fakes record what they were
asked under `$FAKE/<name>`, read with `sb.read()`. The fake `ssh-keygen -L`
prints local time without a zone, as the real one does.

The tests run in parallel threads of one process. A fork in one thread can
inherit a file another thread has open for writing, and running that file
then fails with `ETXTBSY`. The sandbox writes its executables with `cp`, in a
child process, so no fork of the test process ever holds one open.

## The fake release and end to end

`tests/install.rs` builds a release directory whose "binary" is a shell
script that answers `--version`, and points `AICENTRE_DOWNLOAD_BASE` at it.
It checks that upgrade switches versions and removes the old one and the
download, that an unsigned release or a bad checksum changes nothing, and
that uninstall removes everything of aicentre's own but the
settings. `~/.cache` and `~/.local/state` existed before, so only aicentre's
directories inside them go.

With `AICENTRE_E2E=1` it builds two release binaries that report different
versions (`AICENTRE_BUILD_VERSION`), into one target directory under
`CARGO_TARGET_TMPDIR`. It installs the first through `install.sh`, upgrades
to the second, then uninstalls. It takes about a minute, so it is opt-in.

The install tests sign their fake releases with a key they make, so they
need a real `ssh-keygen`. The launcher tests need `python3` on the PATH: the gate is Python, and so is
the stand-in server. The real-sshd test also needs `ssh` and `sshd`.

The launcher tests build on Linux only (`#![cfg(target_os = "linux")]`).
`launcher.sh` runs on the cluster's compute and login nodes, never on the
researcher's machine. On macOS the gate cannot resolve the login node, because
there is no `getent`, and the Nix build sandbox there has neither `ss` nor
`lsof` for the port check.

## CI and Nix

`.github/workflows/cli.yaml` runs rustfmt, clippy and the tests with
`AICENTRE_E2E=1` on Linux and macOS (macOS without the launcher tests) for
every change under `pkgs/cli`, and
shellcheck on Linux. Every release also runs `scripts/install-check.sh`
against its real downloads ([07](07-install-and-release.md#release)).

The Nix package runs the tests in its check phase, without
`AICENTRE_E2E`, with `python3`, `ps` and `ssh-keygen` added. The build
sandbox cannot resolve a host name, not even an IP address through `getent`,
so it skips the two launcher tests whose gate resolves `LSB_SUB_HOST`.
