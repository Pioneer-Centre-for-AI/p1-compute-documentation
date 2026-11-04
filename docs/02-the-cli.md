# The CLI

**Status: work in progress.** The tool is mainly illustrative for now. From our
position we can only really wrap stuff. So it's an idea of what researchers
might need in terms of tooling from the Danish compute ecosystem. The
integrations, tooling and platforms behind it are the providers' to build on
their side; this is a place to show what they could look like from a
researcher's chair. Commands may change, and parts have only been tested
against fake clusters (`test/helpers/sandbox.ts`), not the real ones.

`aicentre` is the command line tool for P1 compute, in `pkgs/cli`. People
install it with

```sh
curl -fsSL https://hpc.aicentre.dk/install | sh
```

and read about it at <https://hpc.aicentre.dk/cli/>
(`src/routes/cli/+page.svx`). It is TypeScript, run and compiled with bun. A
release is one standalone executable per platform, built with
`bun build --compile`, plus a `pi/` folder with the files Pi reads at run time
for `aicentre chat` ([09 · Chat](09-chat.md)). Nothing else is needed to run
it.

## Develop

From the repository root, `nix develop` gives bun and shellcheck. Then, in
`pkgs/cli`:

```sh
bun install
bun run dev check                # bun run src/main.ts check
bun run dev check -p dtu --connect
bun run typecheck                # tsc --noEmit
bun run format                   # prettier --write
bun run lint                     # prettier --check
bun test                         # see 12 · Testing
AICENTRE_E2E=1 bun test
bun run lint:sh                  # shellcheck install.sh and the launcher
bun run build                    # dist/aicentre for this machine
```

`scripts/build.ts` takes `--target` (such as `bun-darwin-arm64`), `--outfile`
(default `dist/aicentre`) and `--version`. The tests use `--version` to build
a binary that reports another version. The version is baked in from
`package.json` as the `AICENTRE_VERSION` constant, so `aicentre --version` and
the release tag agree. A dev run through `bun run src/main.ts` has no such
constant and reports `0.0.0-dev`. The executable is built with
`autoloadDotenv` and `autoloadBunfig` off: a `.env` or `bunfig.toml` in the
directory it runs in belongs to the user's project. Bun rejects an explicit
`target: undefined`, so the key is left out to build for this machine.

`context.ts` decides whether it runs compiled. Bun 1.4 has
`Bun.isStandaloneExecutable`. Older bun (nixpkgs pinned 1.3) only shows it as
`$bunfs` in the embedded module path.

## Layout

| Path | What |
| --- | --- |
| `src/main.ts` | Command dispatch and exit codes |
| `src/context.ts` | `Ctx`: everything a command needs from outside |
| `src/cli/` | Argument parsing and help text |
| `src/config.ts` | `~/.config/aicentre/config.toml` |
| `src/check/` | `aicentre check` ([06](06-check.md)) |
| `src/check/clusters.ts` | Host names, ports and Host block templates per cluster |
| `src/remote/` | The shared ssh login, and running commands here or there ([03](03-providers-and-login.md)) |
| `src/session/` | Login, sessions, the launcher, sync ([03](03-providers-and-login.md), [04](04-sessions.md), [08](08-sync-and-transfer.md)) |
| `src/batch/`, `src/jobs/` | `submit`, `logs`, `cancel`, `list jobs` ([05](05-batch-and-jobs.md)) |
| `src/transfer/` | `push` and `pull` ([08](08-sync-and-transfer.md)) |
| `src/chat/` | `aicentre chat` ([09](09-chat.md)) |
| `src/gallery/` | `aicentre gallery` ([11](11-the-gallery.md)) |
| `src/install/`, `src/platform.ts` | `self-install`, `upgrade`, `uninstall` ([07](07-install-and-release.md)) |
| `src/ui/` | Terminal output |
| `install.sh` | The installer served at `/install` |
| `scripts/build.ts` | `bun build --compile` with the version baked in |
| `nix/` | Package and modules ([10](10-nix.md)) |
| `test/` | Tests and the sandbox ([12](12-testing.md)) |

When a cluster changes a host, a port or its login steps, update
`src/check/clusters.ts` together with the provider page under
`src/routes/providers/` and the gallery's `pkgs/gallery/rem-lgn/ssh_config`.

## Conventions

Exit codes: 0 done, 1 something failed, 2 the command line or the settings
were wrong, 130 interrupted.

`Ctx` is passed in rather than read from globals. `main.ts` builds the real
one from the process. Tests build one around a temporary HOME. It carries the
environment, the output streams, the XDG paths, the uid, the local login name,
whether a person can answer on stdin (`interactive`), the running executable
and whether it is compiled.

Parsing has two steps. The first word picks the command, then that command's
options are parsed strictly, so a typo is an error. Commands work across
every provider this machine is set up for, so a provider is rarely named.
`--provider`/`-p` narrows a command: `-p dtu,gefion` or the flag twice. Some
commands take at most one. A provider typed where an argument goes
(`aicentre logs dtu`) gets a hint to use `-p`. Everything after `chat` is
passed to Pi.

Help pages are data in `src/cli/help.ts`, drawn the same way: a one-line
summary, usage, aligned two-column sections, and a short note wrapped to 78
columns. One column width spans all sections, so Interactive, Batch and Setup
line up. `aicentre` with no arguments prints help. Colour only on a terminal.

`src/exec.ts` runs other programs with a hard time limit and stdin closed, so
nothing can stop to ask a question. With `detached` the child gets its own
session. It cannot open the terminal, and on timeout the whole process group
is killed, including anything ssh's ProxyCommand started. Ctrl-C kills every
child still running. Commands for a login node are wrapped in `bash -lc`,
because the schedulers are only on PATH in a login shell.

`config.toml` is optional, and the installer never writes it. Tables are
`[dtu]`, `[gefion]` and `[ngc]`. Each takes `user`, `alias`, `account` (the
Slurm account on Gefion, `iu_0096` by default) and `sync` (`false` stops
[sync](08-sync-and-transfer.md) to that provider). An empty table still marks
the provider as used, so `check` covers it. Unknown tables or keys are errors.

## Terminal output

- Colour: `NO_COLOR` wins, then `FORCE_COLOR` (not `0`), then `TERM=dumb`
  turns it off, then whether output is a terminal.
- `TERM=dumb` or `AICENTRE_ASCII=1` swaps the symbols and spinner for ASCII.
- `ui/say.ts` writes `aicentre: <message>` and indented hints to stderr and
  returns 1.
- `ui/prompt.ts`: anything but `y` or `yes` is no.
- `ui/live.ts` is a redrawn area at the bottom of the normal screen. Printed
  lines stay in the scrollback, and only the area is cleared and redrawn. It
  never reads stdin and never clears the scrollback, so a fast exit or a
  resize leaves the terminal as it was. When the area is taller than the
  screen it shows the end, because cursor-up would hit the top.
- `ui/steps.ts` shows progress for a command that moves through steps. On a
  terminal the current step spins with its elapsed time. Elsewhere each step
  is one plain line.
- `ui/table.ts` measures widths from the plain text, so a cell can be coloured
  after it is measured.
- `browser.ts` opens a URL in the default browser when there is one and never
  fails the caller, since the URL is on screen too. URLs are OSC 8 links on a
  terminal.
