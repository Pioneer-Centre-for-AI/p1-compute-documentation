import { afterEach, beforeEach, expect, test } from 'bun:test';
import { main } from '../src/main';
import { gefionReady, sandbox, sshG, type Sandbox } from './helpers/sandbox';

let sb: Sandbox;
beforeEach(() => {
  sb = sandbox();
  sb.file('.ssh/config', '# managed by the test\n');
});
afterEach(() => sb.done());

type Check = { id: string; status: string; detail?: string; hint?: string[] };
type Doc = { ok: boolean; groups: { id: string; status: string; checks: Check[] }[] };

async function check(...args: string[]) {
  const code = await main(['check', '--json', ...args], sb.ctx());
  const doc = JSON.parse(sb.out.text) as Doc;
  const result = (id: string) => doc.groups.flatMap((g) => g.checks).find((c) => c.id === id)!;
  return { code, doc, result };
}

test('a ready DTU setup passes', async () => {
  sb.answer(
    'ssh-G-dtu',
    sshG({
      host: 'dtu',
      hostname: 'login9.hpc.dtu.dk',
      user: 's123456',
      port: '22',
      serveraliveinterval: '60',
      identityfile: ['~/.ssh/id_rsa', '~/.ssh/id_ed25519'],
      userknownhostsfile: `${sb.home}/.ssh/known_hosts`
    })
  );
  sb.file('.ssh/id_ed25519', 'key');
  sb.answer('known-login9.hpc.dtu.dk', '');
  const { code, doc, result } = await check();
  expect(code).toBe(0);
  expect(doc.groups.find((g) => g.id === 'dtu')?.status).toBe('ok');
  expect(result('dtu.key').detail).toBe('~/.ssh/id_ed25519');
});

test('when login9 does not answer, a plain check warns but still passes', async () => {
  sb.answer(
    'ssh-G-dtu',
    sshG({
      host: 'dtu',
      hostname: 'login9.hpc.dtu.dk',
      user: 's123456',
      port: '22',
      identityfile: ['~/.ssh/id_ed25519'],
      userknownhostsfile: `${sb.home}/.ssh/known_hosts`
    })
  );
  sb.file('.ssh/id_ed25519', 'key');
  sb.answer('known-login9.hpc.dtu.dk', '');
  sb.answer('tcp-login9.hpc.dtu.dk-22', 'no answer from login9.hpc.dtu.dk:22 within 3 s');
  const { code, result } = await check();
  expect(code).toBe(0);
  expect(result('dtu.reach').status).toBe('warn');
  expect(result('dtu.reach').hint?.[0]).toContain('port 22');
});

test('a provider named without a Host block fails, with the block to add', async () => {
  const { code, doc, result } = await check('-p', 'dtu');
  expect(code).toBe(1);
  expect(doc.groups.map((g) => g.id)).toEqual(['local', 'dtu']);
  expect(result('dtu.host').status).toBe('fail');
  expect(result('dtu.host').hint?.join('\n')).toContain('HostName login9.hpc.dtu.dk');
});

test('an expired Teleport login fails with the command to fix it', async () => {
  gefionReady(sb, new Date(Date.now() - 3_600_000));
  const { code, result } = await check('-p', 'gefion');
  expect(code).toBe(1);
  expect(result('gefion.login')).toMatchObject({ status: 'fail', detail: 'expired 1 h ago' });
  expect(result('gefion.login').hint?.[0]).toBe(
    'tsh login --proxy secureremote.dcai.dk --user jdoe'
  );
});
