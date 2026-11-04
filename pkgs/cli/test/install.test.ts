import { afterEach, beforeEach, describe, expect, test } from 'bun:test';
import {
  existsSync,
  lstatSync,
  mkdirSync,
  readdirSync,
  readlinkSync,
  writeFileSync
} from 'node:fs';
import { join } from 'node:path';
import { run } from '../src/exec';
import { placeBinary, readManifest } from '../src/install/layout';
import { commit } from '../src/install/self-install';
import { uninstall } from '../src/install/uninstall';
import { upgrade } from '../src/install/upgrade';
import { sandbox, type Sandbox } from './helpers/sandbox';

let sb: Sandbox;
beforeEach(() => {
  sb = sandbox();
});
afterEach(() => sb.done());

const ASSET = 'aicentre-linux-x64.tar.gz';

async function release(version: string, opts: { corrupt?: boolean } = {}): Promise<string> {
  const dir = join(sb.root, `rel-${version}`);
  mkdirSync(join(dir, 'stage'), { recursive: true });
  writeFileSync(join(dir, 'stage/aicentre'), `#!/bin/sh\necho ${version}\n`, { mode: 0o755 });
  await run('tar', ['-czf', join(dir, ASSET), '-C', join(dir, 'stage'), 'aicentre'], {
    env: sb.env
  });
  writeFileSync(join(dir, 'VERSION'), `${version}\n`);
  const hex = new Bun.CryptoHasher('sha256')
    .update(await Bun.file(join(dir, ASSET)).bytes())
    .digest('hex');
  writeFileSync(join(dir, 'SHA256SUMS'), `${hex}  ${ASSET}\n`);
  if (opts.corrupt) writeFileSync(join(dir, ASSET), 'not a tarball');
  return dir;
}

function installed() {
  const ctx = sb.ctx();
  const fake = join(sb.root, 'fake-0.1.0');
  writeFileSync(fake, '#!/bin/sh\necho 0.1.0\n', { mode: 0o755 });
  const { created } = placeBinary(ctx.paths, '0.1.0', fake);
  commit(ctx, '0.1.0', created);
  return ctx;
}

function tree(): string[] {
  const out: string[] = [];
  const walk = (dir: string, rel: string) => {
    for (const name of readdirSync(dir)) {
      const r = rel ? `${rel}/${name}` : name;
      out.push(r);
      if (lstatSync(join(dir, name)).isDirectory()) walk(join(dir, name), r);
    }
  };
  walk(sb.home, '');
  return out.sort();
}

const from = (base: string) => sb.ctx({ env: { ...sb.env, AICENTRE_DOWNLOAD_BASE: base } });

test('upgrade switches to the new version and removes the old one and the download', async () => {
  const ctx = installed();
  const code = await upgrade(from(await release('0.2.0')), {});
  expect(sb.err.text).toBe('');
  expect(code).toBe(0);
  expect(readlinkSync(ctx.paths.current)).toBe('versions/0.2.0');
  expect(readdirSync(ctx.paths.versions)).toEqual(['0.2.0']);
  expect(readManifest(ctx.paths)?.version).toBe('0.2.0');
  expect(existsSync(ctx.paths.cache)).toBe(false);
});

test('an upgrade with a bad checksum changes nothing', async () => {
  const ctx = installed();
  const before = tree();
  expect(await upgrade(from(await release('0.2.0', { corrupt: true })), {})).toBe(1);
  expect(sb.err.text).toContain('does not match its checksum');
  expect(tree()).toEqual(before);
  expect(readlinkSync(ctx.paths.current)).toBe('versions/0.1.0');
});

test('uninstall removes everything of its own but the settings', async () => {
  const ctx = installed();
  sb.file('.config/aicentre/config.toml', '[dtu]\nuser = "s1"\n');
  sb.file('.local/state/aicentre/last-check', 'x');
  sb.file('.cache/aicentre/leftover', 'x');
  mkdirSync(ctx.paths.runtime, { recursive: true });
  expect(await uninstall(ctx, { purge: false })).toBe(0);
  expect(existsSync(ctx.paths.runtime)).toBe(false);
  expect(tree()).toEqual([
    '.cache',
    '.config',
    '.config/aicentre',
    '.config/aicentre/config.toml',
    '.local',
    '.local/state'
  ]);
  expect(sb.out.text).toContain('Kept ~/.config/aicentre');
});

describe.if(process.env.AICENTRE_E2E === '1')('end to end', () => {
  test('install.sh, upgrade, uninstall', async () => {
    const arch = process.arch === 'arm64' ? 'arm64' : 'x64';
    const os = process.platform === 'darwin' ? 'darwin' : 'linux';
    const asset = `aicentre-${os}-${arch}.tar.gz`;
    const cli = join(import.meta.dir, '..');
    const env = {
      ...sb.env,
      PATH: `${sb.env.PATH}:${process.env.PATH}`,
      NIX_LD: process.env.NIX_LD
    };
    for (const v of ['0.1.0', '0.2.0']) {
      const dir = join(sb.root, `real-${v}`);
      const build = await run(
        'bun',
        ['run', join(cli, 'scripts/build.ts'), '--version', v, '--outfile', join(dir, 'aicentre')],
        { env: process.env, timeoutMs: 120_000 }
      );
      expect(build.code).toBe(0);
      await run('tar', ['-czf', join(dir, asset), '-C', dir, 'aicentre', 'pi'], { env });
      writeFileSync(join(dir, 'VERSION'), `${v}\n`);
      const hex = new Bun.CryptoHasher('sha256')
        .update(await Bun.file(join(dir, asset)).bytes())
        .digest('hex');
      writeFileSync(join(dir, 'SHA256SUMS'), `${hex}  ${asset}\n`);
    }
    const install = await run('sh', [join(cli, 'install.sh')], {
      env: { ...env, AICENTRE_DOWNLOAD_BASE: join(sb.root, 'real-0.1.0') },
      timeoutMs: 60_000
    });
    expect(install.stderr).toBe('');
    expect(install.code).toBe(0);
    const bin = join(sb.home, '.local/bin/aicentre');
    expect((await run(bin, ['--version'], { env })).stdout.trim()).toBe('0.1.0');

    const up = await run(bin, ['upgrade'], {
      env: { ...env, AICENTRE_DOWNLOAD_BASE: join(sb.root, 'real-0.2.0') },
      timeoutMs: 60_000
    });
    expect(up.code).toBe(0);
    expect((await run(bin, ['--version'], { env })).stdout.trim()).toBe('0.2.0');
    expect(readdirSync(join(sb.home, '.local/share/aicentre/versions'))).toEqual(['0.2.0']);
    expect(readdirSync(join(sb.home, '.local/share/aicentre/versions/0.2.0/pi'))).toContain(
      'theme'
    );

    sb.file('.config/aicentre/config.toml', '[dtu]\nuser = "s1"\n');
    expect((await run(bin, ['uninstall'], { env })).code).toBe(0);
    expect(tree().filter((p) => p.includes('aicentre'))).toEqual([
      '.config/aicentre',
      '.config/aicentre/config.toml'
    ]);
  }, 180_000);
});
