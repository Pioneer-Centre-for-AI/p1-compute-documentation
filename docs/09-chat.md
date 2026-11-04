# Chat

`aicentre chat` runs Pi's interactive agent (the `@earendil-works/pi-coding-agent`
SDK) with P1 tools and P1 instructions (`src/chat/`). Everything after `chat`
on the command line goes to Pi: `--model`, `--continue`, a first message. It
needs a terminal and exits 2 without one.

The model is whatever the user sets up in Pi (`/login`, or a provider's API
key in the environment). aicentre has no opinion. Pi's own files live under
aicentre's XDG directories rather than `~/.pi`, so `aicentre uninstall`
accounts for them:

- settings and logins in `~/.config/aicentre/agent`, through
  `PI_CODING_AGENT_DIR`;
- conversations in `~/.local/state/aicentre/chat`, through `--session-dir`.

aicentre calls Pi's `main(args, { extensionFactories })` with
`--tools read,grep,find,ls,write,edit,p1_*` and `--append-system-prompt`. The
agent may read files and write drafts, such as job scripts, in the working
directory. It gets no shell. What it does on P1 goes through the `p1_*`
tools.

## The tools

`src/chat/tools.ts` registers the tools with `pi.registerTool` and TypeBox
schemas. Each one runs the aicentre command a person would type, with its
output captured instead of printed, so the agent sees what the user would
see.

| Tool | Does |
| --- | --- |
| `p1_docs` | Fetches `llms.txt` or one page's `.md` twin from the site |
| `p1_jobs` | `list jobs` |
| `p1_check` | `check` |
| `p1_logs` | `logs` |
| `p1_submit` | `submit`, after a yes |
| `p1_cancel` | `cancel`, after a yes |

Looking is free. Acting asks first: `p1_submit` and `p1_cancel` show what
they are about to do and wait for a yes through `ctx.ui.confirm`. Without a
chat UI to ask in there is no yes. Nothing in the tools prompts for an ssh
password, since the chat owns the terminal. A provider that wants one is
logged in to from another terminal (any aicentre command does it), and that
login is shared.

## Pi's files in a compiled build

Pi reads its themes, docs and `package.json` from files at run time. A
compiled build cannot read them from `node_modules`, so `scripts/build.ts`
copies them into `pi/` next to the executable: `package.json`, `README.md`,
`CHANGELOG.md`, `docs/`, and the interactive theme, assets and HTML export
directories. Without them Pi reports version 0.0.0. The release ships `pi/` in
the tarball, `upgrade` keeps it next to the binary, and `chat` sets
`PI_PACKAGE_DIR` to it when it is not already set. Through a symlink, such as
a Nix profile's `bin/`, the files are next to the real binary, so the path
goes through `realpath`.
