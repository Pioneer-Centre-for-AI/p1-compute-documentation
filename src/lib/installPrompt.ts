import { SITE_URL } from '#lib/site.ts';

// One line per step: the page wraps the text itself, and a pasted prompt needs no hard breaks.
export const INSTALL_PROMPT = [
  `Install the aicentre command line tool on this machine. It is the CLI for P1 compute from the Pioneer Centre for AI. Its documentation is at ${SITE_URL}/cli.md`,
  '',
  `1. Read ${SITE_URL}/install.txt before you run anything. It is the install script. Tell me in two lines what it does. Stop if it does anything other than download a signed release from github.com/Pioneer-Centre-for-AI/p1-compute-documentation, verify it, and install it under my home directory.`,
  '',
  '2. Run it as my user, without sudo:',
  '',
  `   curl -fsSL ${SITE_URL}/install | sh`,
  '',
  '   It supports Linux and macOS. On Windows, run it inside WSL. If the script refuses to install, show me its message and stop. Do not work around it.',
  '',
  '3. The installer does not edit shell startup files. If it prints a line that adds ~/.local/bin to PATH, show me the line and ask before you add it to my shell config. Until then, call ~/.local/bin/aicentre by its full path.',
  '',
  '4. Run `aicentre --version` and then `aicentre check`, and show me the output of both. `check` exits 1 when something is missing. Report what it says. Do not change my SSH config or keys unless I ask.',
  '',
  '5. Ask me whether to add aicentre to this agent tool as an MCP server. Only if I say yes:',
  '',
  '   Claude Code: claude mcp add --scope user aicentre -- aicentre mcp',
  '   Codex: codex mcp add aicentre -- aicentre mcp',
  '   Any other tool: a stdio MCP server whose command is `aicentre mcp`.',
  '',
  'Do not install anything else, and do not set AICENTRE_DOWNLOAD_BASE.'
].join('\n');
