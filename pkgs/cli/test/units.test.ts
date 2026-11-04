import { describe, expect, test } from 'bun:test';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { directiveWords, schedulerOf, sets } from '../src/batch/script';
import { aliasesFor } from '../src/check/aliases';
import { classifySshFailure, type FailureContext } from '../src/check/probes/connect';
import { isConfigured, parseSshG, words } from '../src/check/probes/ssh';
import { parseCommand, UsageError } from '../src/cli/args';
import { renderHelp } from '../src/cli/help';
import { ConfigError, parseConfig } from '../src/config';
import type { Ctx, Env } from '../src/context';
import { parseBjobs } from '../src/jobs/lsf';
import { parseSqueue } from '../src/jobs/slurm';
import { resolvePaths } from '../src/paths';
import { decide } from '../src/session/sync';
import { rsyncFlags } from '../src/transfer/transfer';
import { makeStyle } from '../src/ui/style';

test('paths follow XDG under HOME; absolute variables win, relative ones are ignored', () => {
  const p = resolvePaths({ HOME: '/h' }, 501);
  expect([p.data, p.config, p.state, p.cache, p.binLink, p.runtime]).toEqual([
    '/h/.local/share/aicentre',
    '/h/.config/aicentre',
    '/h/.local/state/aicentre',
    '/h/.cache/aicentre',
    '/h/.local/bin/aicentre',
    '/tmp/aicentre-501'
  ]);
  const q = resolvePaths(
    { HOME: '/h', XDG_DATA_HOME: '/d', XDG_CONFIG_HOME: 'relative', XDG_RUNTIME_DIR: '/run/u' },
    501
  );
  expect([q.data, q.config, q.runtime]).toEqual([
    '/d/aicentre',
    '/h/.config/aicentre',
    '/run/u/aicentre'
  ]);
});

test('ssh -G output, and telling a configured host from the echo of an unknown one', () => {
  const cfg = parseSshG(
    'host dtu\nhostname dtu\nidentityfile ~/.ssh/id_rsa\nidentityfile ~/.ssh/id_ed25519\nuserknownhostsfile /a /b\n'
  );
  expect(cfg.get('identityfile')).toEqual(['~/.ssh/id_rsa', '~/.ssh/id_ed25519']);
  expect(words(cfg, 'userknownhostsfile')).toEqual(['/a', '/b']);
  expect(isConfigured(cfg, 'dtu')).toBe(false);
  expect(isConfigured(parseSshG('hostname login9.hpc.dtu.dk\n'), 'dtu')).toBe(true);
});

test('ssh failures are classified from stderr', () => {
  const c: FailureContext = {
    alias: 'dtu',
    host: 'login9.hpc.dtu.dk',
    viaTsh: false,
    timedOut: false,
    timeoutHint: ['VPN'],
    teleportHint: ['tsh login']
  };
  const cases: [string, string, Partial<FailureContext>?][] = [
    ['ssh: Could not resolve hostname login9.hpc.dtu.dk: Name or service not known', 'dns'],
    ['ssh: connect to host x port 22: Connection timed out', 'timeout'],
    ['ssh: connect to host x port 22: Connection refused', 'refused'],
    [
      'WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!\nHost key verification failed.',
      'host-key-changed'
    ],
    ['Host key verification failed.', 'host-key-unknown'],
    ['s1@x: Permission denied (publickey,gssapi-keyex).', 'key-refused'],
    ['s1@x: Permission denied (keyboard-interactive).', 'password'],
    [
      "WARNING: UNPROTECTED PRIVATE KEY FILE!\nPermissions 0644 for '/h/.ssh/id' are too open.",
      'unprotected-key'
    ],
    ['ERROR: Active profile expired.', 'teleport', { viaTsh: true }],
    ['', 'timeout', { timedOut: true }],
    ['something new', 'other']
  ];
  for (const [stderr, kind, extra] of cases)
    expect(classifySshFailure(stderr, { ...c, ...extra }).kind, stderr).toBe(kind);
  expect(classifySshFailure('ERROR: Active profile expired.', c).kind).not.toBe('teleport');
});

test('config.toml takes known providers and settings only', () => {
  expect(parseConfig('[dtu]\nuser = "s1"\n[gefion]\naccount = "p1-team"\n', 'c')).toEqual({
    dtu: { user: 's1' },
    gefion: { account: 'p1-team' }
  });
  for (const text of ['[lumi]\nuser = "x"\n', '[dtu]\nname = "x"\n', '[dtu]\nuser = 3\n', '[dtu\n'])
    expect(() => parseConfig(text, 'c'), text).toThrow(ConfigError);
});

describe('arguments', () => {
  test('commands', () => {
    expect(parseCommand([])).toEqual({ name: 'help', topic: undefined });
    expect(parseCommand(['check', '-p', 'gefion,NGC', '--connect'])).toEqual({
      name: 'check',
      clusters: ['gefion', 'ngc'],
      connect: true,
      json: false
    });
    expect(parseCommand(['update', '--version', '0.2.0'])).toEqual({
      name: 'upgrade',
      version: '0.2.0'
    });
    expect(parseCommand(['marimo', 'nb.py', '--gpu', '--time', '2h', '--no-browser'])).toEqual({
      name: 'session',
      session: {
        app: 'marimo',
        path: 'nb.py',
        gpu: true,
        provider: undefined,
        server: 'vscode',
        minutes: 120,
        cpus: 4,
        memPerCpu: '8GB',
        browser: false
      }
    });
    expect(parseCommand(['logs', '123', '-f', '-n', '200'])).toMatchObject({
      id: '123',
      follow: true,
      lines: 200
    });
    expect(parseCommand(['pull', 'results', '.', '-p', 'gefion'])).toEqual({
      name: 'transfer',
      direction: 'pull',
      from: 'results',
      to: '.',
      provider: 'gefion'
    });
    expect(parseCommand(['gallery', 'get', 'int-gpu', 'mine'])).toMatchObject({
      gallery: { action: 'get', code: 'int-gpu', dir: 'mine' }
    });
    expect(parseCommand(['chat', '--model', 'x', '-c'])).toEqual({
      name: 'chat',
      args: ['--model', 'x', '-c']
    });
    expect(parseCommand(['code', '--help'])).toEqual({ name: 'help', topic: 'code' });
  });

  test('mistakes are usage errors', () => {
    for (const argv of [
      ['check', '-p', 'lumi'],
      ['check', '--all', '-p', 'dtu'],
      ['check', '--conect'],
      ['list'],
      ['shell', '--time', '2h'],
      ['code', '--gpu', '--mem', 'lots'],
      ['shell', '--no-browser'],
      ['code', '-p', 'dtu,gefion'],
      ['submit', 'a', 'b'],
      ['logs', 'abc'],
      ['cancel'],
      ['push'],
      ['gallery', 'fetch', 'x'],
      ['gpu', 'shell'],
      ['frobnicate']
    ])
      expect(() => parseCommand(argv), argv.join(' ')).toThrow(UsageError);
    expect(() => parseCommand(['list', 'jobs', 'gefion'])).toThrow('add --provider gefion');
  });

  test('help fits 80 columns and has no colour when piped', () => {
    const plain = makeStyle({}, { write: () => true, isTTY: false });
    for (const topic of [undefined, 'check', 'list', 'marimo', 'submit', 'gallery']) {
      const text = renderHelp(topic, plain, '0.1.0');
      expect(text).not.toContain('\x1b[');
      for (const line of text.split('\n')) expect(line.length, line).toBeLessThanOrEqual(80);
    }
  });
});

test('bjobs output', () => {
  const jobs = parseBjobs(
    [
      '123|RUN|p1|train|4*n-62-30-1:4*n-62-30-2|3725 second(s)|70:35 L|-',
      '124|PEND|p1|sweep[3]|-|0 second(s)|-|Not enough GPUs available: 2 hosts;',
      'not a job line'
    ].join('\n')
  );
  expect(jobs).toEqual([
    {
      id: '123',
      state: 'running',
      rawState: 'RUN',
      queue: 'p1',
      name: 'train',
      node: 'n-62-30-1',
      reason: '',
      elapsed: '1:02:05',
      left: '2-22:35:00'
    },
    {
      id: '124',
      state: 'pending',
      rawState: 'PEND',
      queue: 'p1',
      name: 'sweep[3]',
      node: '',
      reason: 'Not enough GPUs available: 2 hosts;',
      elapsed: '',
      left: ''
    }
  ]);
});

test('squeue output', () => {
  const jobs = parseSqueue(
    [
      '98765|RUNNING|p1|train|dgx-[01-02]|1-02:03:04|22:56:56|None',
      '98767_[1-4]|PENDING|p1|sweep||0:00|UNLIMITED|(Priority)'
    ].join('\n')
  );
  expect(jobs[0]).toMatchObject({
    id: '98765',
    state: 'running',
    node: 'dgx-01',
    left: '22:56:56',
    reason: ''
  });
  expect(jobs[1]).toMatchObject({
    id: '98767_[1-4]',
    state: 'pending',
    left: '',
    reason: '(Priority)'
  });
});

test('a job script names its scheduler, and what it sets is left to it', () => {
  const lsf = '#!/bin/bash\n#BSUB -J train\n#BSUB -W 2:00  # two hours\npython train.py\n';
  expect(schedulerOf(lsf)).toBe('lsf');
  expect(schedulerOf('#SBATCH --gres=gpu:1\n')).toBe('slurm');
  expect(schedulerOf('#PBS -l walltime=1:00:00\n')).toBe('pbs');
  expect(schedulerOf('#!/bin/bash\npython x.py\n')).toBeNull();
  const w = directiveWords(lsf, 'lsf');
  expect(w).toEqual(['-J', 'train', '-W', '2:00']);
  expect(sets(w, ['-W'])).toBe(true);
  expect(sets(w, ['-o', '-oo'])).toBe(false);
  expect(sets(directiveWords('#SBATCH --account=p1\n', 'slurm'), ['-A', '--account'])).toBe(true);
});

test('Host names are found by HostName, through Include, in file order', () => {
  const home = mkdtempSync(join(tmpdir(), 'aicentre-alias-'));
  try {
    mkdirSync(join(home, '.ssh/config.d'), { recursive: true });
    writeFileSync(
      join(home, '.ssh/config'),
      [
        'Host first',
        '  HostName login9.hpc.dtu.dk',
        'Include ~/.ssh/config.d/* missing/*',
        'Host other',
        '  HostName=login9.hpc.dtu.dk # comment'
      ].join('\n')
    );
    writeFileSync(
      join(home, '.ssh/config.d/a'),
      'Host "gbar.p1" p1 *.wild !neg\n  hostname LOGIN9.hpc.dtu.dk\nInclude ../config\n'
    );
    expect(aliasesFor('login9.hpc.dtu.dk', home)).toEqual(['first', 'gbar.p1', 'p1', 'other']);
    expect(aliasesFor('slurm-login-01.hpc.ite.dcai.dk', home)).toEqual([]);
  } finally {
    rmSync(home, { recursive: true, force: true });
  }
});

test('rsync flags fit the rsync on this machine', () => {
  expect(rsyncFlags('rsync  version 3.4.1  protocol version 32')).toEqual([
    '-az',
    '--partial',
    '--protect-args',
    '--info=progress2'
  ]);
  const old = ['-az', '--partial', '--progress'];
  expect(rsyncFlags('rsync  version 2.6.9  protocol version 29')).toEqual(old);
  expect(rsyncFlags('openrsync: protocol version 29')).toEqual(old);
});

test('aicentre is installed on a cluster when missing or older, never downgraded, never from a dev build', () => {
  const ctx = (version: string, env: Env = {}) => ({ version, env }) as Ctx;
  expect(decide(ctx('0.2.0'), {}, 'dtu', '')).toBe('install');
  expect(decide(ctx('0.2.0'), {}, 'dtu', '0.1.0')).toBe('install');
  expect(decide(ctx('0.2.0'), {}, 'dtu', '0.2.0')).toBe('none');
  expect(decide(ctx('0.2.0'), {}, 'dtu', '0.3.0')).toBe('newer-there');
  expect(decide(ctx('0.0.0-dev'), {}, 'dtu', '')).toBe('off');
  expect(decide(ctx('0.2.0'), { dtu: { sync: false } }, 'dtu', '')).toBe('off');
  expect(decide(ctx('0.2.0', { AICENTRE_SYNC: '0' }), {}, 'gefion', '')).toBe('off');
});
