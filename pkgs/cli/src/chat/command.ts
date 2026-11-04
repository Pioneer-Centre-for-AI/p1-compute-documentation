import { existsSync, mkdirSync, realpathSync } from 'node:fs';
import { dirname, join } from 'node:path';
import type { Ctx } from '../context';
import { P1_PROMPT, p1Extension } from './tools';

export const TOOLS = ['read', 'grep', 'find', 'ls', 'write', 'edit', 'p1_*'];

export function chatArgs(ctx: Ctx, passthrough: string[]): string[] {
  return [
    '--session-dir',
    join(ctx.paths.state, 'chat'),
    '--tools',
    TOOLS.join(','),
    '--append-system-prompt',
    P1_PROMPT,
    ...passthrough
  ];
}

export async function chat(ctx: Ctx, passthrough: string[]): Promise<number> {
  if (!ctx.interactive) {
    ctx.err.write('aicentre: chat needs a terminal to talk in.\n');
    return 2;
  }
  const agentDir = join(ctx.paths.config, 'agent');
  mkdirSync(agentDir, { recursive: true, mode: 0o700 });
  mkdirSync(join(ctx.paths.state, 'chat'), { recursive: true });
  process.env.PI_CODING_AGENT_DIR = agentDir;
  const assets = join(
    dirname(existsSync(ctx.execPath) ? realpathSync(ctx.execPath) : ctx.execPath),
    'pi'
  );
  if (ctx.compiled && !process.env.PI_PACKAGE_DIR && existsSync(assets))
    process.env.PI_PACKAGE_DIR = assets;
  const { main } = await import('@earendil-works/pi-coding-agent');
  await main(chatArgs(ctx, passthrough), {
    extensionFactories: [{ name: 'p1', factory: p1Extension(ctx) }]
  });
  return 0;
}
