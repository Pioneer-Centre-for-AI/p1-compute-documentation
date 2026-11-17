# The CLI

**Status: work in progress.** The tool is mainly illustrative for now. From our
position we can only really wrap stuff. So it's an idea of what researchers
might need in terms of tooling from the Danish compute ecosystem. The
integrations, tooling and platforms behind it are the providers' to build on
their side; this is a place to show what they could look like from a
researcher's chair. Commands may change, and parts have only been tested
against fake clusters (`tests/common/mod.rs`), not the real ones.

`aicentre` is the command line tool for P1 compute, in `pkgs/cli`. People
install it with

```sh
curl -fsSL https://hpc.aicentre.dk/install | sh
```

and read about it at <https://hpc.aicentre.dk/cli/>
(`src/routes/cli/+page.svx`). It is Rust, one crate with a library and a thin
binary. A release is one executable per platform: on Linux it links musl
statically, so it runs on any distribution and on NixOS. Nothing else is
needed to run it. Through 0.1 it was TypeScript, compiled with bun.

## Develop

From the repository root, `nix develop` gives cargo, rustc, clippy, rustfmt
and shellcheck. Then, in `pkgs/cli`:

```sh
cargo run -- check               # a dev run
cargo run -- check -p dtu --connect
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test                       # see 12 · Testing
AICENTRE_E2E=1 cargo test
cargo build --release            # target/release/aicentre for this machine
shellcheck -s sh install.sh
shellcheck -s bash src/session/launcher.sh scripts/install-check.sh
```

The version is `version` in `Cargo.toml`, so `aicentre --version` and the
release tag agree. `AICENTRE_BUILD_VERSION` set at build time overrides it;
the end-to-end test uses it to build a binary that reports another version.

A debug build counts as a dev run and a release build as an installed
release (`compiled` in `Ctx`, from `debug_assertions`). `self-install` and
`upgrade` refuse a dev run.

## Layout

| Path | What |
| --- | --- |
| `src/main.rs` | Builds the `Ctx` and the runtime, exits with the code |
| `src/lib.rs` | `run`: command dispatch and exit codes |
| `src/context.rs` | `Ctx`: everything a command needs from outside |
| `src/cli/` | Argument parsing and help text |
| `src/config.rs` | `~/.config/aicentre/config.toml` |
| `src/check/` | `aicentre check` ([06](06-check.md)) |
| `src/check/clusters.rs` | Host names, ports and Host block templates per cluster |
| `src/remote/` | The shared ssh login, and running commands here or there ([03](03-providers-and-login.md)) |
| `src/session/` | Login, sessions, the launcher, sync ([03](03-providers-and-login.md), [04](04-sessions.md), [08](08-sync-and-transfer.md)) |
| `src/jobs/` | `list jobs` and `list gpus` ([05](05-jobs.md)) |
| `src/transfer/` | `push` and `pull` ([08](08-sync-and-transfer.md)) |
| `src/mcp/` | `aicentre mcp` and the P1 tools ([09](09-mcp.md)) |
| `src/gallery/` | `aicentre gallery` ([11](11-the-gallery.md)) |
| `src/install/`, `src/platform.rs` | `self-install`, `upgrade`, `uninstall` ([07](07-install-and-release.md)) |
| `src/exec.rs` | Running other programs |
| `src/http.rs` | Downloads, with rustls and the system's certificates; an https URL never redirects to http, and a body stops at 256 MB |
| `src/ui/` | Terminal output |
| `install.sh` | The installer served at `/install` |
| `nix/` | Package and modules ([10](10-nix.md)) |
| `tests/` | Tests and the sandbox ([12](12-testing.md)) |

When a cluster changes a host, a port or its login steps, update
`src/check/clusters.rs` together with the provider page under
`src/routes/providers/` and the gallery's `pkgs/gallery/rem-lgn/ssh_config`.

## Conventions

Exit codes: 0 done, 1 something failed, 2 the command line or the settings
were wrong, 130 interrupted.

`Ctx` is passed in rather than read from globals. `main.rs` builds the real
one from the process. Tests build one around a temporary HOME. It carries the
environment, the output streams, the XDG paths, the uid, the local login name,
whether a person can answer on stdin (`interactive`), the running executable
and whether it is compiled.

Parsing has two steps. The first word picks the command, then that command's
options are parsed strictly, so a typo is an error. Commands work across
every provider this machine is set up for, so a provider is rarely named.
`--provider`/`-p` narrows a command: `-p dtu,gefion` or the flag twice. Some
commands take at most one. A provider typed where an argument goes
(`aicentre list jobs dtu`) gets a hint to use `-p`. `chat`, `gpu`, `submit`,
`logs` and `cancel` are gone and say what to use instead.

The parser in `src/cli/args.rs` reads options the way node's
`util.parseArgs` did in 0.1, with the same messages: `-fn5` is `-f -n 5`,
`--lines=5` and `-n5` carry their value, and `--` ends the options.

Help pages are data in `src/cli/help.rs`, drawn the same way: a one-line
summary, usage, aligned two-column sections, and a short note wrapped to 78
columns. One column width spans all sections, so Interactive, Batch and Setup
line up. `aicentre` with no arguments prints help. Colour only on a terminal.

`src/exec.rs` runs other programs with a hard time limit and stdin closed, so
nothing can stop to ask a question. With `detached` the child gets its own
session. It cannot open the terminal, and on timeout the whole process group
is killed, including anything ssh's ProxyCommand started. Ctrl-C kills every
child still running. Commands for a login node are wrapped in `bash -lc`,
because the schedulers are only on PATH in a login shell.

`config.toml` is optional, and the installer never writes it. Tables are
`[dtu]`, `[gefion]` and `[ngc]`. Each takes `user`, `alias` and `sync`
(`false` stops [sync](08-sync-and-transfer.md) to that provider). `account`,
the Slurm account 0.1's `submit` used, is still accepted and does nothing, so
an older file still loads. An empty table still marks
the provider as used, so `check` covers it. `[desktop]` takes `open`, the
command that opens a remote folder in the user's editor, with `{host}` in it
(see [desktop editors](04-sessions.md#desktop-editors)). Unknown tables or keys
are errors.

## Terminal output

- Colour: `NO_COLOR` wins, then `FORCE_COLOR` (not `0`), then `TERM=dumb`
  turns it off, then whether output is a terminal.
- `TERM=dumb` or `AICENTRE_ASCII=1` swaps the symbols and spinner for ASCII.
- On a terminal, the main help page opens with the Pioneer Centre for AI mark
  reduced to its 3 by 3 grid: seven `▀` cells, the top-right and bottom-left
  empty. An upper half block in one cell is square, and the lower half is the
  gap. The centre cell is the site's coral (`#fd5061`, or colour 203 on a
  256-colour terminal) when colour is on. Piped output, `TERM=dumb`,
  `AICENTRE_ASCII=1` and every other help page keep the one-line header, so
  scripts, agents and screen readers do not get the art.
- `ui/say.rs` writes `aicentre: <message>` and indented hints to stderr and
  returns 1. A usage error that already starts with the command, such as
  `aicentre list needs something to list`, is printed without the prefix, so
  the name is not said twice.
- `ui/table.rs` pads columns to their widest cell. Columns named in `right`
  are right-aligned, which `list gpus` uses for numbers.
- `--verbose` anywhere on the command line, or `AICENTRE_DEBUG=1`, prints
  every program aicentre runs to stderr as `[aicentre] <command>  (exit 0,
  0.42 s)`: the ssh, rsync and editor commands, quoted as a shell would take
  them. A long-running one is printed when it starts and again when it ends.
  An argument over 240 characters is cut, with the number of characters
  left out. Text sent on stdin is not shown. The lines are dim on a terminal
  and plain when stderr is piped. `aicentre proxy` ignores the flag, since
  ssh passes it a host name, but honours the variable. This is the first
  thing to ask for in a bug report.
- `ui/prompt.rs`: anything but `y` or `yes` is no.
- `ui/live.rs` is a redrawn area at the bottom of the normal screen. Printed
  lines stay in the scrollback, and only the area is cleared and redrawn. It
  never reads stdin and never clears the scrollback, so a fast exit or a
  resize leaves the terminal as it was. When the area is taller than the
  screen it shows the end, because cursor-up would hit the top.
- `ui/steps.rs` shows progress for a command that moves through steps. On a
  terminal the current step spins with its elapsed time. Elsewhere each step
  is one plain line.
- `ui/table.rs` measures widths from the plain text, so a cell can be coloured
  after it is measured.
- `browser.rs` opens a URL in the default browser when there is one and never
  fails the caller, since the URL is on screen too. URLs are OSC 8 links on a
  terminal.
