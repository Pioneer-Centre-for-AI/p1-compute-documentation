# MCP

`aicentre mcp` is an MCP server on stdio. An agent tool the user already has
starts it and gets the P1 tools: the docs, the user's jobs and their setup.
All three only read.

```sh
claude mcp add --scope user aicentre -- aicentre mcp     # Claude Code
codex mcp add aicentre -- aicentre mcp                   # Codex
```

Any other agent tool that takes an MCP server on stdio runs the command
`aicentre mcp`.

## No chat since 0.2

0.1 had `aicentre chat`, which ran Pi, a TypeScript agent, inside the CLI with
the same tools. The CLI is Rust since 0.2 and cannot carry Pi, and an agent
tool the user already has does the same work with their own model login.
`aicentre chat` now exits 2 and points to `aicentre mcp`.

0.1 kept Pi's settings and model logins in `~/.config/aicentre/agent`, and its
conversations in `~/.local/state/aicentre/chat`. 0.2 reads neither.
`uninstall` removes the state directory, and `uninstall --purge` removes the
config directory with the logins in it. The user manual tells users who had
chat to delete `~/.config/aicentre/agent` by hand.

## The tools

`P1Tools` in `src/mcp/tools.rs` is the list of tools, each with a JSON
Schema for its arguments. Each tool runs the aicentre command a person would
type, with its output captured instead of printed, so the agent sees what the
user would see. The captured output is not a terminal, 100 columns wide, with
`NO_COLOR=1`, and `interactive` is false.

| Tool | Does |
| --- | --- |
| `p1_docs` | Fetches `llms.txt` or one page's `.md` twin from the site |
| `p1_jobs` | `list jobs` |
| `p1_check` | `check` |

0.1 also had `p1_logs`, `p1_submit` and `p1_cancel`, with the commands they
ran ([05](05-jobs.md)). With them went every tool that acts, and with that
the need to ask the user anything. The instructions tell the agent that the
user submits, follows and cancels batch jobs in `aicentre shell`, and that it
can write a job script for them and copy it up with `aicentre push`.

An agent may call a tool many times in a row, and each `p1_jobs` or
`p1_check` call is a login and a scheduler query on a shared login node. So
those two run one at a time, and an answer that succeeded is given again for
60 seconds to the same question. An answer with a failure in it is kept for
15 seconds. On DTU, with Gefion's login expired, every `p1_jobs` call failed
in part, nothing was kept, and each call reached DTU again. Repeated failed
logins are also what a login node's ban rules count. After 15 seconds a
retry asks the cluster again, so a user who logs in waits that long at
most.

`check` in a terminal rewrites aicentre's own `ssh_config`
([04](04-sessions.md#on-a-gpu-node)). Through `p1_check` it does not, so the
tool reads only, as its `readOnlyHint` says.

Nothing in the tools prompts for an ssh password, since stdin and stdout
carry the protocol. A provider that wants one is logged in to from a terminal
(any aicentre command does it), and that login is shared.

## The protocol

`src/mcp/server.rs` speaks newline-delimited JSON-RPC 2.0 on stdin and stdout,
without an SDK. It answers `initialize`, `ping`, `tools/list` and
`tools/call`. Of the notifications it acts only on `notifications/cancelled`,
and ignores the rest. It accepts protocol versions `2024-11-05` to
`2025-11-25` and answers with the client's own when it knows it. Its
`instructions` carry the facts about the providers and the rules for the
tools.

The schemas are written out as JSON. They are the JSON that 0.1's TypeBox
schemas gave, so clients see no change. A small checker covers the parts of
JSON Schema they use, and words its errors as TypeBox did. A wrong argument
is a tool result with `isError`, and an unknown tool is JSON-RPC error
-32602. Every tool carries `readOnlyHint` and `idempotentHint`.

Each request runs in its own task, and the loop that reads stdin never waits
for one. A `tools/call` the client cancels still runs to its end, but gets no
response, as the MCP spec asks. The server asks the client nothing, so it
declares no client features and ignores messages that have no method.

stdout carries only protocol. The tools write to a captured output, and their
ssh runs with stdin closed and `BatchMode=yes`. Started from a terminal,
`aicentre mcp` refuses and exits 2. When stdin closes, the server waits for
the calls in flight and exits 0.
