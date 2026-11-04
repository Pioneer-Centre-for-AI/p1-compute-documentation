import {
  copyFileSync,
  chmodSync,
  cpSync,
  existsSync,
  lstatSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  readlinkSync,
  renameSync,
  rmSync,
  rmdirSync,
  symlinkSync,
  unlinkSync,
  writeFileSync
} from 'node:fs';
import { dirname, isAbsolute, join, resolve } from 'node:path';
import { run } from '../exec';
import type { Env } from '../context';
import type { Paths } from '../paths';
import type { Platform } from '../platform';

export type Manifest = {
  schema: 1;
  version: string;
  os: string;
  arch: string;
  binLink: string;
  installedAt: string;
  updatedAt: string;
  createdDirs: string[];
};

export function readManifest(paths: Paths): Manifest | null {
  try {
    const m = JSON.parse(readFileSync(paths.manifest, 'utf8')) as Manifest;
    return m.schema === 1 ? m : null;
  } catch {
    return null;
  }
}

function writeAtomic(path: string, text: string): void {
  const tmp = `${path}.tmp-${process.pid}`;
  writeFileSync(tmp, text);
  renameSync(tmp, path);
}

export function writeManifest(
  paths: Paths,
  version: string,
  platform: Platform,
  created: string[] = []
): void {
  const now = new Date().toISOString();
  const previous = readManifest(paths);
  const outside = (d: string) => d !== paths.data && !d.startsWith(paths.data + '/');
  const manifest: Manifest = {
    schema: 1,
    version,
    os: platform.os,
    arch: platform.arch,
    binLink: paths.binLink,
    installedAt: previous?.installedAt ?? now,
    updatedAt: now,
    createdDirs: [...new Set([...(previous?.createdDirs ?? []), ...created.filter(outside)])]
  };
  writeAtomic(paths.manifest, JSON.stringify(manifest, null, 2) + '\n');
}

export function versionDir(paths: Paths, version: string): string {
  return join(paths.versions, version);
}

export function binaryPath(paths: Paths, version: string): string {
  return join(versionDir(paths, version), 'aicentre');
}

export function mkdirp(path: string): string[] {
  const created: string[] = [];
  for (let p = path; !existsSync(p); p = dirname(p)) created.unshift(p);
  mkdirSync(path, { recursive: true });
  return created;
}

export function removeIfEmpty(dirs: string[]): void {
  for (const dir of [...dirs].sort((a, b) => b.length - a.length)) {
    try {
      rmdirSync(dir);
    } catch {}
  }
}

export function placeBinary(
  paths: Paths,
  version: string,
  source: string
): { path: string; created: string[] } {
  const dir = versionDir(paths, version);
  const created = mkdirp(dir);
  const target = join(dir, 'aicentre');
  const tmp = `${target}.tmp-${process.pid}`;
  copyFileSync(source, tmp);
  chmodSync(tmp, 0o755);
  renameSync(tmp, target);
  const assets = join(dirname(source), 'pi');
  if (existsSync(assets)) {
    const staged = join(dir, `pi.tmp-${process.pid}`);
    rmSync(staged, { recursive: true, force: true });
    cpSync(assets, staged, { recursive: true });
    rmSync(join(dir, 'pi'), { recursive: true, force: true });
    renameSync(staged, join(dir, 'pi'));
  }
  return { path: target, created };
}

export async function smokeTest(binary: string, version: string, env: Env): Promise<string | null> {
  const r = await run(binary, ['--version'], { env, timeoutMs: 10_000 });
  const got = r.stdout.trim();
  if (r.code !== 0 || got !== version) {
    return `${binary} --version printed "${got || r.stderr.trim()}" (exit ${r.code}), expected "${version}"`;
  }
  return null;
}

function swapLink(target: string, link: string): void {
  const tmp = `${link}.tmp-${process.pid}`;
  rmSync(tmp, { force: true });
  symlinkSync(target, tmp);
  renameSync(tmp, link);
}

export function activate(paths: Paths, version: string): void {
  swapLink(join('versions', version), paths.current);
}

export function isOurLink(link: string, paths: Paths): boolean {
  try {
    if (!lstatSync(link).isSymbolicLink()) return false;
    const target = readlinkSync(link);
    const full = isAbsolute(target) ? target : resolve(dirname(link), target);
    return full === paths.data || full.startsWith(paths.data + '/');
  } catch {
    return false;
  }
}

export type LinkResult = { result: 'created' | 'kept' | 'foreign'; created: string[] };

export function linkBin(paths: Paths): LinkResult {
  const target = join(paths.current, 'aicentre');
  let exists = true;
  try {
    lstatSync(paths.binLink);
  } catch {
    exists = false;
  }
  if (exists && !isOurLink(paths.binLink, paths)) return { result: 'foreign', created: [] };
  if (exists && readlinkSync(paths.binLink) === target) return { result: 'kept', created: [] };
  const created = mkdirp(paths.binDir);
  swapLink(target, paths.binLink);
  return { result: 'created', created };
}

export function safeRemove(path: string, paths: Paths): void {
  const roots = [paths.data, paths.config, paths.state, paths.cache, paths.runtime];
  const inside = [paths.data, paths.cache].some((root) => path.startsWith(root + '/'));
  if (!isAbsolute(path) || !(roots.includes(path) || inside)) {
    throw new Error(`refusing to remove ${path}: aicentre does not own it`);
  }
  rmSync(path, { recursive: true, force: true });
}

export function pruneVersions(paths: Paths, keep: string): string[] {
  let entries: string[] = [];
  try {
    entries = readdirSync(paths.versions);
  } catch {
    return [];
  }
  const removed: string[] = [];
  for (const name of entries) {
    if (name === keep) continue;
    safeRemove(join(paths.versions, name), paths);
    removed.push(name);
  }
  return removed;
}

export function removeLink(path: string): void {
  unlinkSync(path);
}
