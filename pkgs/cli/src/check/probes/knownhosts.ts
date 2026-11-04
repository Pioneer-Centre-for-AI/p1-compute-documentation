import { run } from '../../exec';
import type { Env } from '../../context';

export async function knownHost(
  host: string,
  port: number,
  files: string[],
  env: Env
): Promise<string | null> {
  const name = port === 22 ? host : `[${host}]:${port}`;
  for (const file of files) {
    const r = await run('ssh-keygen', ['-F', name, '-f', file], { env, timeoutMs: 5_000 });
    if (r.code === 0) return file;
  }
  return null;
}
