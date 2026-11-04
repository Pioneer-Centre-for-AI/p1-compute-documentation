import { existsSync } from 'node:fs';
import { isAbsolute } from 'node:path';
import type { Env } from '../context';

export const REPO = 'https://github.com/Pioneer-Centre-for-AI/p1-compute-documentation';

export function downloadBase(env: Env, version?: string): string {
  if (env.AICENTRE_DOWNLOAD_BASE) return env.AICENTRE_DOWNLOAD_BASE.replace(/\/+$/, '');
  return version ? `${REPO}/releases/download/cli-v${version}` : `${REPO}/releases/latest/download`;
}

function localPath(base: string): string | null {
  if (base.startsWith('file://')) return new URL(base).pathname;
  return isAbsolute(base) ? base : null;
}

export class DownloadError extends Error {}

async function open(base: string, name: string): Promise<Blob> {
  const dir = localPath(base);
  if (dir) {
    const path = `${dir}/${name}`;
    if (!existsSync(path)) throw new DownloadError(`${path} does not exist`);
    return Bun.file(path);
  }
  const url = `${base}/${name}`;
  let res: Response;
  try {
    res = await fetch(url, { redirect: 'follow', signal: AbortSignal.timeout(120_000) });
  } catch (error) {
    throw new DownloadError(`cannot download ${url}: ${(error as Error).message}`);
  }
  if (!res.ok) throw new DownloadError(`cannot download ${url}: HTTP ${res.status}`);
  return res.blob();
}

export async function fetchText(base: string, name: string): Promise<string> {
  return (await open(base, name)).text();
}

export async function download(base: string, name: string, dest: string): Promise<void> {
  await Bun.write(dest, await open(base, name));
}

export function parseSums(text: string): Map<string, string> {
  const sums = new Map<string, string>();
  for (const line of text.split('\n')) {
    const m = /^([0-9a-f]{64})\s+\*?(\S+)$/i.exec(line.trim());
    if (m) sums.set(m[2]!, m[1]!.toLowerCase());
  }
  return sums;
}

export async function sha256(path: string): Promise<string> {
  const hasher = new Bun.CryptoHasher('sha256');
  for await (const chunk of Bun.file(path).stream()) hasher.update(chunk);
  return hasher.digest('hex');
}

const SEMVER = /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/;

export function isVersion(v: string): boolean {
  return SEMVER.test(v);
}
