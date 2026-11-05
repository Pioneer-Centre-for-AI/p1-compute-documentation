# Testing

```sh
cd pkgs/cli
bun test                  # unit and scenario tests
AICENTRE_E2E=1 bun test   # also the end-to-end install run
bun run lint:sh           # shellcheck -s sh install.sh, -s bash src/session/launcher.sh
```

The tests cover core behaviour only. They are kept few on purpose.

| File | Covers |
| --- | --- |
| `test/units.test.ts` | Pure functions: paths, parsing, classification, the decisions commands make |
| `test/check.test.ts` | `aicentre check` end to end, against fake ssh, ssh-keygen, ssh-add and tsh |
| `test/remote.test.ts` | Commands that reach a provider, against a fake ssh that plays the login node, LSF and Slurm: sessions, batch jobs, the job list, the gallery, push, and chat's tools |
| `test/install.test.ts` | `upgrade` and `uninstall` against a fake release, and with `AICENTRE_E2E=1` the real thing |

## The sandbox

The tests never touch the developer's own `~/.ssh`. OpenSSH takes the home
directory from the password database rather than `$HOME`, so a real ssh would
read the real `~/.ssh` whatever the test sets. `test/helpers/sandbox.ts`
makes a throwaway HOME with fake `ssh`, `ssh-keygen`, `ssh-add` and `tsh`
first on a PATH that holds only the few real tools the code needs (`sh`,
`bash`, `tar`, `gzip`, `cat`, `env`, `sleep`, `touch`, `rm`, `mkdir`). The
checks only ever meet the fakes.

The fakes are shell scripts driven by files in `$FAKE/answers/<name>`, which a
test writes with `sb.answer()`. `ssh -G <alias>` prints
`answers/ssh-G-<alias>`, or the default for an unconfigured host. The fake ssh
plays the shared login too: `ControlMaster=yes` touches `$FAKE/master.up`,
which stands for an open login, and `-O check` tests for it. For a login probe,
the alias comes right before the remote command, and the answers are
`probe-<alias>.stdout`, `.stderr` and `.code`. The fakes record what they were
asked under `$FAKE/<name>`, read with `sb.read()`. The fake `ssh-keygen -L`
prints local time without a zone, as the real one does.

## The fake release and end to end

`install.test.ts` builds a release directory whose "binary" is a shell script
that answers `--version`, and points `AICENTRE_DOWNLOAD_BASE` at it. It
checks that upgrade switches versions and removes the old one and the
download, that a bad checksum changes nothing, and that uninstall removes
everything of aicentre's own but the settings. `~/.cache` and `~/.local/state`
existed before, so only aicentre's directories inside them go.

With `AICENTRE_E2E=1` it compiles two real versions (`build --version`),
installs the first through `install.sh`, upgrades to the second, then
uninstalls. It also checks that Pi's files came along for chat. It is slow
(two bun compiles), so it is opt-in.

`bunfig.toml` sets the test timeout to 20 seconds. Scenario tests run many
fake ssh round trips, and under the parallel compile of the end-to-end run 5
seconds is too tight.

`.github/workflows/cli.yaml` runs the typecheck, prettier, the tests and
shellcheck on Linux and macOS for every change under `pkgs/cli`. Every release also runs
`scripts/install-check.sh` against its real downloads
([07](07-install-and-release.md#release)).
