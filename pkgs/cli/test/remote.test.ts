import { afterEach, beforeEach, describe, expect, test } from 'bun:test';
import { chmodSync, existsSync, mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import type { ExtensionAPI } from '@earendil-works/pi-coding-agent';
import { findRecord } from '../src/batch/records';
import { p1Extension } from '../src/chat/tools';
import { main } from '../src/main';
import { interactive, type InteractiveRequest, type SessionDeps } from '../src/session/interactive';
import { dtuHost, gefionReady, sandbox, type Sandbox } from './helpers/sandbox';

let sb: Sandbox;
beforeEach(() => {
  sb = sandbox();
  sb.file('.ssh/config', '');
  dtuHost(sb);
});
afterEach(() => sb.done());

const LSF_SCRIPT = '#!/bin/bash\n#BSUB -J train\n#BSUB -W 2:00\npython train.py\n';
const SLURM_SCRIPT = '#!/bin/bash\n#SBATCH --job-name=train\npython train.py\n';
const inHours = (h: number) => new Date(Date.now() + h * 3_600_000);

function script(name: string, text: string): string {
  const path = join(sb.root, name);
  writeFileSync(path, text);
  return path;
}

describe('sessions', () => {
  const REQUEST: InteractiveRequest = {
    app: 'marimo',
    path: '',
    gpu: true,
    server: 'vscode',
    minutes: 60,
    cpus: 4,
    memPerCpu: '8GB',
    browser: true
  };

  function deps() {
    const controller = new AbortController();
    const opened: string[] = [];
    const d: SessionDeps = {
      signal: controller.signal,
      freePort: async () => 55555,
      openUrl: async (url) => {
        opened.push(url);
        setTimeout(() => controller.abort(), 50);
      }
    };
    return { d, opened };
  }

  test('a GPU session gets ready, then Ctrl-C ends the job and cancels the forward', async () => {
    sb.answer(
      'bsub.out',
      [
        'Job <123> is submitted to queue <p1i>.',
        '<<Starting on n-62-30-1>>',
        '@@PID 777',
        '@@HOST n-62-30-1',
        '@@PORT 40123',
        '@@PATH /?access_token=abc',
        ''
      ].join('\n')
    );
    sb.answer('bsub.hold', '');
    const { d, opened } = deps();
    expect(await interactive(sb.ctx(), REQUEST, d)).toBe(130);
    expect(sb.err.text).toBe('');
    expect(opened).toEqual(['http://127.0.0.1:55555/?access_token=abc']);
    expect(sb.out.text).toContain('marimo is ready on n-62-30-1');
    expect(sb.out.text).toContain('Ended job 123.');
    expect(sb.read('bsub')).toContain(
      'marimo /dtu/p1/s123456 vscode /dtu/p1/s123456/.aicentre /dtu/p1/s123456 0.0.0.0'
    );
    expect(sb.read('bkill')).toContain('bkill 123');
    expect(sb.read('cancel')).toContain('-O cancel -L 127.0.0.1:55555:n-62-30-1:40123 dtu');
    expect(existsSync(join(sb.fake, 'master.up'))).toBe(true);
  });

  test('a CPU session runs on the login node and forwards to its 127.0.0.1', async () => {
    sb.answer('login.out', '@@PID 4242\n@@HOST login9\n@@PORT 40999\n@@PATH /?tkn=xyz\n');
    sb.answer('login.hold', '');
    const { d, opened } = deps();
    const code = await interactive(sb.ctx(), { ...REQUEST, app: 'code', gpu: false }, d);
    expect(sb.err.text).toBe('');
    expect(code).toBe(130);
    expect(opened).toEqual(['http://127.0.0.1:55555/?tkn=xyz']);
    expect(sb.out.text).toContain('VS Code is ready on dtu, the P1 DTU HPC login node');
    expect(sb.read('calls')).toContain('-O forward -L 127.0.0.1:55555:127.0.0.1:40999 dtu');
    expect(sb.read('kill')).toBe('kill 4242\n');
    expect(sb.read('bsub')).toBe('');
  });
});

describe('batch', () => {
  test('an LSF script goes to DTU, into queue p1, with its output where logs looks', async () => {
    sb.answer('submit.out', 'Job <4242> is submitted to queue <p1>.\n');
    expect(await main(['submit', script('train.sh', LSF_SCRIPT)], sb.ctx())).toBe(0);
    expect(sb.err.text).toBe('');
    expect(sb.out.text).toContain('Submitted job 4242 to P1 DTU HPC (queue p1)');
    expect(sb.read('submit')).toContain(
      'cd /dtu/p1/s123456 && bsub -q p1 -o /dtu/p1/s123456/.aicentre/logs/%J.out -e /dtu/p1/s123456/.aicentre/logs/%J.err <'
    );
    expect(findRecord(sb.ctx().paths, '4242')).toMatchObject({
      provider: 'dtu',
      out: '/dtu/p1/s123456/.aicentre/logs/4242.out'
    });
  });

  test("a Slurm script goes to Gefion, charged to P1's account iu_0096", async () => {
    gefionReady(sb, inHours(5));
    sb.answer('sbatch.out', '98765\n');
    expect(
      await main(['submit', script('train.sh', SLURM_SCRIPT), '--dir', 'proj'], sb.ctx())
    ).toBe(0);
    expect(sb.err.text).toBe('');
    expect(sb.out.text).toContain('Submitted job 98765 to P1 Gefion HPC');
    expect(sb.read('sbatch')).toContain(
      'sbatch --parsable --chdir=/home/s123456/proj --account=iu_0096 --output=/home/s123456/.aicentre/logs/%j.out'
    );
  });

  test('cancel goes to the provider that ran each job', async () => {
    gefionReady(sb, inHours(5));
    sb.answer('submit.out', 'Job <4242> is submitted to queue <p1>.\n');
    sb.answer('sbatch.out', '98765\n');
    await main(['submit', script('a.sh', LSF_SCRIPT)], sb.ctx());
    await main(['submit', script('b.sh', SLURM_SCRIPT)], sb.ctx());
    expect(await main(['cancel', '4242', '98765'], sb.ctx())).toBe(0);
    expect(sb.read('bkill')).toContain('bkill 4242');
    expect(sb.read('scancel')).toContain('scancel 98765');
    expect(findRecord(sb.ctx().paths, '4242')).toBeNull();
  });
});

describe('list jobs', () => {
  test('every provider in one table, running first wherever it runs', async () => {
    gefionReady(sb, inHours(5));
    sb.answer(
      'bjobs',
      '124|PEND|p1|sweep|-|0 second(s)|-|Not enough GPUs\n123|RUN|p1i|aicentre-marimo|4*n-62-30-1|300 second(s)|0:55 L|-\n'
    );
    sb.answer('squeue', '98765|RUNNING|p1|train|dgx-01|2:03:04|21:56:56|None\n');
    expect(await main(['list', 'jobs'], sb.ctx())).toBe(0);
    expect(sb.err.text).toBe('');
    const out = sb.out.text;
    expect(out).toContain('Your jobs on P1 compute  3 jobs');
    const order = ['123', '98765', '124'].map((id) => out.indexOf(`  ${id}  `));
    expect(order).toEqual([...order].sort((a, b) => a - b));
    expect(out).toMatch(/DTU\s+124\s+pending\s+p1\s+sweep\s+-\s+-\s+-\s+Not enough GPUs/);
    expect(out).toMatch(/Gefion\s+98765\s+running\s+p1\s+train\s+dgx-01\s+2:03:04\s+21:56:56/);
  });

  test('a password prompt: log in on this terminal once, then ask again through that login', async () => {
    sb.answer('needs-login', '');
    sb.answer('bjobs', '123|RUN|p1|train|1*n-62-30-1|60 second(s)|0:59 L|-\n');
    expect(await main(['list', 'jobs'], sb.ctx({ interactive: true }))).toBe(0);
    expect(sb.out.text).toContain('dtu asks for more than your key. Log in here');
    expect(sb.out.text).toMatch(/DTU\s+123\s+running/);
    sb.out.text = '';
    expect(await main(['list', 'jobs'], sb.ctx({ interactive: true }))).toBe(0);
    expect(sb.out.text).not.toContain('Log in here');
  });
});

test('gallery get has the login node fetch the example, then says what to run', async () => {
  const site = join(sb.root, 'site');
  mkdirSync(join(site, 'gallery'), { recursive: true });
  const example = {
    code: 'INT-GPU',
    slug: 'int-gpu',
    title: 'Interactive session with a GPU',
    summary: '',
    providers: ['dtu'],
    page: 'https://hpc.aicentre.dk/gallery/int-gpu/',
    files: 'https://hpc.aicentre.dk/gallery/int-gpu.tar.gz',
    commands: ['aicentre marimo examples/int-gpu/notebook.py --gpu']
  };
  writeFileSync(join(site, 'gallery/index.json'), JSON.stringify({ examples: [example] }));
  const ctx = sb.ctx({ env: { ...sb.env, AICENTRE_SITE: `file://${site}` } });
  expect(await main(['gallery', 'get', 'INT-GPU'], ctx)).toBe(0);
  expect(sb.read('calls')).toContain(
    'mkdir -p /dtu/p1/s123456/examples && { curl -fsSL https://hpc.aicentre.dk/gallery/int-gpu.tar.gz || wget -qO-'
  );
  expect(sb.out.text).toContain('INT-GPU is in /dtu/p1/s123456/examples/int-gpu on P1 DTU HPC');
  expect(sb.out.text).toContain('$ aicentre marimo examples/int-gpu/notebook.py --gpu');
});

test('push copies into the project directory with rsync, through the shared login', async () => {
  const rsync = join(sb.fake, 'bin', 'rsync');
  writeFileSync(
    rsync,
    '#!/bin/sh\n[ "$1" = --version ] && { echo "rsync  version 3.4.1  protocol version 32"; exit 0; }\nprintf "%s\\n" "$@" > "$FAKE/rsync"\n'
  );
  chmodSync(rsync, 0o755);
  expect(await main(['push', 'datasets/', 'data'], sb.ctx())).toBe(0);
  const args = sb.read('rsync').trim().split('\n');
  expect(args.slice(-2)).toEqual(['datasets', 'dtu:/dtu/p1/s123456/data/']);
  expect(args[args.indexOf('-e') + 1]).toMatch(/^ssh -S \S+cm-[0-9a-f]{12} -o ControlMaster=no$/);
  expect(sb.read('mkdir')).toContain('mkdir -p /dtu/p1/s123456/data/');
});

test("the chat's p1_submit asks first, and does nothing without a yes", async () => {
  type Tool = {
    name: string;
    execute: (...args: unknown[]) => Promise<{ content: { text: string }[] }>;
  };
  const tools = new Map<string, Tool>();
  const pi = { registerTool: (t: Tool) => tools.set(t.name, t), on: () => {} };
  await p1Extension(sb.ctx())(pi as unknown as ExtensionAPI);
  const path = script('train.sh', LSF_SCRIPT);
  const submit = async (answer: boolean, hasUI = true) => {
    const ui = { hasUI, ui: { confirm: async () => answer } };
    const r = await tools
      .get('p1_submit')!
      .execute('id', { script: path }, undefined, undefined, ui);
    return r.content[0]!.text;
  };
  sb.answer('submit.out', 'Job <4242> is submitted to queue <p1>.\n');
  expect(await submit(false)).toBe('The user did not submit the job.');
  expect(await submit(true, false)).toBe('The user did not submit the job.');
  expect(sb.read('submit')).toBe('');
  expect(await submit(true)).toContain('Submitted job 4242 to P1 DTU HPC');
});
