import type { Ctx } from '../context';

export function fail(ctx: Pick<Ctx, 'err'>, message: string, hints: string[] = []): 1 {
  ctx.err.write(`aicentre: ${message}\n${hints.map((h) => `  ${h}\n`).join('')}`);
  return 1;
}
