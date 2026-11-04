import {
  closeSync,
  existsSync,
  mkdirSync,
  openSync,
  readFileSync,
  renameSync,
  rmSync,
  writeSync
} from 'node:fs';
import { join } from 'node:path';
import type { Ctx } from '../context';
import { run } from '../exec';
import { ensureRuntimeDir, tilde } from '../paths';
import { assetName, isNixStore } from '../platform';
import { fail } from '../ui/say';
import { makeStyle } from '../ui/style';
import { mkdirp, readManifest, removeIfEmpty, safeRemove, smokeTest, versionDir } from './layout';
import {
  download,
  downloadBase,
  fetchText,
  isVersion,
  parseSums,
  sha256,
  DownloadError
} from './release';
import { commit } from './self-install';

const INSTALL = 'curl -fsSL https://hpc.aicentre.dk/install | sh';

function lock(ctx: Ctx): () => void {
  const path = join(ensureRuntimeDir(ctx.paths, ctx.uid), 'upgrade.lock');
  for (let attempt = 0; attempt < 2; attempt++) {
    try {
      const fd = openSync(path, 'wx');
      writeSync(fd, String(process.pid));
      closeSync(fd);
      return () => rmSync(path, { force: true });
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== 'EEXIST') throw error;
      const pid = Number(readFileSync(path, 'utf8'));
      try {
        if (pid) process.kill(pid, 0);
        throw new Error(`another upgrade is running (pid ${pid})`);
      } catch (e) {
        if ((e as NodeJS.ErrnoException).code !== 'ESRCH' && pid) throw e;
        rmSync(path, { force: true });
      }
    }
  }
  throw new Error('cannot take the upgrade lock');
}

export async function upgrade(ctx: Ctx, opts: { version?: string }): Promise<number> {
  const s = makeStyle(ctx.env, ctx.out);

  if (!ctx.compiled)
    return fail(ctx, 'upgrade works on an installed release; this is a development run.', [
      'Update the checkout with git pull instead.'
    ]);
  if (isNixStore(ctx.execPath))
    return fail(ctx, 'this aicentre comes from nix; upgrade it with nix.');
  if (!readManifest(ctx.paths))
    return fail(
      ctx,
      `${tilde(ctx.paths.manifest, ctx.paths.home)} is missing, so this copy was not set up by the installer.`,
      [`Reinstall with: ${INSTALL}`]
    );
  if (opts.version && !isVersion(opts.version))
    return fail(ctx, `"${opts.version}" is not a version like 0.2.0.`);

  const base = downloadBase(ctx.env, opts.version);
  let release: () => void;
  try {
    release = lock(ctx);
  } catch (error) {
    return fail(ctx, (error as Error).message);
  }

  const asset = assetName(ctx.platform);
  const tarball = join(ctx.paths.cache, asset);
  let stage: string | undefined;
  let cacheParents: string[] = [];
  try {
    const target = opts.version ?? (await fetchText(base, 'VERSION')).trim();
    if (!isVersion(target))
      return fail(ctx, `the release names its version "${target}", which is not a version.`);
    const order = Bun.semver.order(target, ctx.version);
    if (order === 0) {
      ctx.out.write(
        `${s.symbol('ok')} aicentre ${ctx.version} is ${opts.version ? 'already installed' : 'the latest'}.\n`
      );
      return 0;
    }
    if (order < 0 && !opts.version) {
      ctx.out.write(
        `${s.symbol('ok')} aicentre ${ctx.version} is newer than the latest release, ${target}.\n`
      );
      return 0;
    }

    ctx.out.write(
      `Downloading aicentre ${target} for ${ctx.platform.os}-${ctx.platform.arch}...\n`
    );
    const sums = parseSums(await fetchText(base, 'SHA256SUMS'));
    const expected = sums.get(asset);
    if (!expected) return fail(ctx, `the ${target} release has no ${asset}.`);
    cacheParents = mkdirp(ctx.paths.cache).filter((d) => d !== ctx.paths.cache);
    await download(base, asset, tarball);
    const actual = await sha256(tarball);
    if (actual !== expected)
      return fail(ctx, `${asset} does not match its checksum; nothing was changed.`, [
        `expected ${expected}, got ${actual}`
      ]);

    stage = `${versionDir(ctx.paths, target)}.partial`;
    safeRemove(stage, ctx.paths);
    mkdirSync(stage, { recursive: true });
    const untar = await run('tar', ['-xzf', tarball, '-C', stage], {
      env: ctx.env,
      timeoutMs: 120_000
    });
    if (untar.code !== 0 || !existsSync(join(stage, 'aicentre')))
      return fail(ctx, `cannot unpack ${asset}: ${untar.stderr.trim() || 'no aicentre inside'}`);
    const dir = versionDir(ctx.paths, target);
    safeRemove(dir, ctx.paths);
    renameSync(stage, dir);
    stage = dir;

    const problem = await smokeTest(join(dir, 'aicentre'), target, ctx.env);
    if (problem)
      return fail(ctx, `the new binary does not run, so nothing was changed: ${problem}`);

    const link = commit(ctx, target);
    stage = undefined;
    ctx.out.write(`${s.symbol('ok')} Upgraded aicentre ${ctx.version} → ${target}\n`);
    if (link === 'foreign')
      ctx.out.write(
        `${s.symbol('warn')} ${tilde(ctx.paths.binLink, ctx.paths.home)} is not ours, so it was left alone.\n`
      );
    return 0;
  } catch (error) {
    if (error instanceof DownloadError) return fail(ctx, error.message);
    throw error;
  } finally {
    if (stage) safeRemove(stage, ctx.paths);
    rmSync(tarball, { force: true });
    if (existsSync(ctx.paths.cache)) safeRemove(ctx.paths.cache, ctx.paths);
    removeIfEmpty(cacheParents);
    release();
  }
}
