import { parseArgs } from 'node:util';
import { CLUSTER_IDS, type ClusterId } from '../check/clusters';
import type { InteractiveRequest } from '../session/interactive';
import type { GalleryRequest } from '../gallery/gallery';
import { isMemory, parseMinutes } from '../session/lsf';

export type Command =
  | { name: 'help'; topic?: string }
  | { name: 'version' }
  | { name: 'check'; clusters: ClusterId[]; connect: boolean; json: boolean }
  | { name: 'upgrade'; version?: string }
  | { name: 'uninstall'; purge: boolean }
  | { name: 'self-install' }
  | { name: 'list'; what: 'jobs'; clusters: ClusterId[]; json: boolean }
  | { name: 'session'; session: InteractiveRequest }
  | { name: 'submit'; script: string; provider?: ClusterId; dir?: string }
  | { name: 'logs'; id: string; follow: boolean; lines: number; err: boolean; provider?: ClusterId }
  | { name: 'cancel'; ids: string[]; provider?: ClusterId }
  | { name: 'chat'; args: string[] }
  | {
      name: 'transfer';
      direction: 'push' | 'pull';
      from: string;
      to?: string;
      provider?: ClusterId;
    }
  | { name: 'gallery'; gallery: GalleryRequest };

export class UsageError extends Error {}

const ALIASES: Record<string, string> = { update: 'upgrade' };

const PROVIDER = { type: 'string', short: 'p', multiple: true } as const;

function providers(value: unknown): ClusterId[] {
  const names = (Array.isArray(value) ? value : [])
    .flatMap((v) => String(v).split(','))
    .map((v) => v.trim().toLowerCase())
    .filter(Boolean);
  for (const n of names) {
    if (!(CLUSTER_IDS as readonly string[]).includes(n))
      throw new UsageError(`unknown provider "${n}"; expected ${CLUSTER_IDS.join(', ')}`);
  }
  return [...new Set(names)] as ClusterId[];
}

function oneProvider(value: unknown): ClusterId | undefined {
  const named = providers(value);
  if (named.length > 1) throw new UsageError('this command works on one provider at a time');
  return named[0];
}

function jobId(text: string): string {
  if (!/^\d+(?:_\d+)?$/.test(text)) throw new UsageError(`"${text}" is not a job id`);
  return text;
}

function noPositionals(command: string, positionals: string[]): void {
  const [first] = positionals;
  if (first === undefined) return;
  if ((CLUSTER_IDS as readonly string[]).includes(first)) {
    throw new UsageError(
      `aicentre ${command} covers every provider you use; to narrow it, add --provider ${first}`
    );
  }
  throw new UsageError(`aicentre ${command} takes no argument "${first}"`);
}

type Options = Record<string, { type: 'string' | 'boolean'; short?: string; multiple?: boolean }>;
type Values = Record<string, string | boolean | string[] | undefined>;

function parse(args: string[], options: Options, positionals: boolean) {
  try {
    return parseArgs({
      args,
      options: { ...options, help: { type: 'boolean', short: 'h' } },
      allowPositionals: positionals,
      strict: true
    }) as { values: Values; positionals: string[] };
  } catch (error) {
    throw new UsageError(
      (error as Error).message.replace(/\. To specify a positional argument.*$/, '')
    );
  }
}

type Spec = {
  options: Options;
  positionals: boolean;
  build(values: Values, args: string[], name: string): Command;
};

const SESSION: Spec = {
  options: {
    gpu: { type: 'boolean' },
    time: { type: 'string' },
    cpus: { type: 'string' },
    mem: { type: 'string' },
    server: { type: 'string' },
    'no-browser': { type: 'boolean' },
    provider: PROVIDER
  },
  positionals: true,
  build(values, args, name) {
    const app = name as InteractiveRequest['app'];
    if (args.length > 1) throw new UsageError(`aicentre ${app} takes one path, not ${args.length}`);
    const gpu = Boolean(values.gpu);
    for (const flag of ['time', 'cpus', 'mem'] as const) {
      if (values[flag] !== undefined && !gpu)
        throw new UsageError(
          `--${flag} sizes a --gpu session; a login-node session runs until you end it`
        );
    }
    const minutes = parseMinutes(String(values.time ?? '60'));
    if (!minutes)
      throw new UsageError(`--time "${values.time}" is not a duration like 90, 2h or 1:30`);
    const cpus = Number(values.cpus ?? 4);
    if (!Number.isInteger(cpus) || cpus < 1 || cpus > 64)
      throw new UsageError(`--cpus "${values.cpus}" is not a number of CPUs`);
    const memPerCpu = String(values.mem ?? '8GB').toUpperCase();
    if (!isMemory(memPerCpu)) throw new UsageError(`--mem "${values.mem}" is not a size like 8GB`);
    const server = String(values.server ?? 'vscode');
    if (server !== 'vscode' && server !== 'openvscode')
      throw new UsageError(`--server "${server}" is not vscode or openvscode`);
    if (values.server && app !== 'code')
      throw new UsageError('--server only applies to aicentre code');
    if (values['no-browser'] && app === 'shell')
      throw new UsageError('--no-browser does not apply to a shell');
    const provider = oneProvider(values.provider);
    const browser = !values['no-browser'];
    const path = args[0] ?? '';
    const session: InteractiveRequest = {
      app,
      path,
      gpu,
      provider,
      server,
      minutes,
      cpus,
      memPerCpu,
      browser
    };
    return { name: 'session', session };
  }
};

const TRANSFER: Spec = {
  options: { provider: PROVIDER },
  positionals: true,
  build(values, args, name) {
    const direction = name as 'push' | 'pull';
    if (args.length < 1 || args.length > 2)
      throw new UsageError(
        direction === 'push'
          ? 'aicentre push takes a local path, and where on the cluster to put it'
          : 'aicentre pull takes a path on the cluster, and where to put it here'
      );
    const provider = oneProvider(values.provider);
    return { name: 'transfer', direction, from: args[0]!, to: args[1], provider };
  }
};

const COMMANDS: Record<string, Spec> = {
  check: {
    options: {
      connect: { type: 'boolean' },
      json: { type: 'boolean' },
      all: { type: 'boolean' },
      provider: PROVIDER
    },
    positionals: true,
    build(values, args) {
      noPositionals('check', args);
      const named = providers(values.provider);
      if (values.all && named.length)
        throw new UsageError('--all already checks every provider; leave out --provider');
      return {
        name: 'check',
        clusters: values.all ? [...CLUSTER_IDS] : named,
        connect: Boolean(values.connect),
        json: Boolean(values.json)
      };
    }
  },
  upgrade: {
    options: { version: { type: 'string' } },
    positionals: false,
    build: (values) => ({ name: 'upgrade', version: values.version as string | undefined })
  },
  uninstall: {
    options: { purge: { type: 'boolean' } },
    positionals: false,
    build: (values) => ({ name: 'uninstall', purge: Boolean(values.purge) })
  },
  list: {
    options: { json: { type: 'boolean' }, provider: PROVIDER },
    positionals: true,
    build(values, [what, ...extra]) {
      if (what !== 'jobs')
        throw new UsageError(
          what
            ? `cannot list "${what}"; try aicentre list jobs`
            : 'aicentre list needs something to list: jobs'
        );
      noPositionals('list jobs', extra);
      return {
        name: 'list',
        what,
        clusters: providers(values.provider),
        json: Boolean(values.json)
      };
    }
  },
  shell: SESSION,
  marimo: SESSION,
  code: SESSION,
  submit: {
    options: { dir: { type: 'string' }, provider: PROVIDER },
    positionals: true,
    build(values, args) {
      if (args.length !== 1) throw new UsageError('aicentre submit takes one job script');
      const provider = oneProvider(values.provider);
      return { name: 'submit', script: args[0]!, provider, dir: values.dir as string | undefined };
    }
  },
  logs: {
    options: {
      follow: { type: 'boolean', short: 'f' },
      lines: { type: 'string', short: 'n' },
      err: { type: 'boolean' },
      provider: PROVIDER
    },
    positionals: true,
    build(values, args) {
      if (args.length !== 1) throw new UsageError('aicentre logs takes one job id');
      const lines = Number(values.lines ?? 50);
      if (!Number.isInteger(lines) || lines < 1)
        throw new UsageError(`-n "${values.lines}" is not a number of lines`);
      return {
        name: 'logs',
        id: jobId(args[0]!),
        follow: Boolean(values.follow),
        lines,
        err: Boolean(values.err),
        provider: oneProvider(values.provider)
      };
    }
  },
  cancel: {
    options: { provider: PROVIDER },
    positionals: true,
    build(values, args) {
      if (args.length === 0)
        throw new UsageError('aicentre cancel takes the ids of the jobs to cancel');
      return { name: 'cancel', ids: args.map(jobId), provider: oneProvider(values.provider) };
    }
  },
  push: TRANSFER,
  pull: TRANSFER,
  gallery: {
    options: { provider: PROVIDER },
    positionals: true,
    build(values, [action, code, dir, ...extra]) {
      if (action === undefined || action === 'list')
        return { name: 'gallery', gallery: { action: 'list' } };
      if (action !== 'get')
        throw new UsageError(`aicentre gallery takes get <code>, not "${action}"`);
      if (!code || extra.length)
        throw new UsageError('aicentre gallery get takes an example code, and where to put it');
      const provider = oneProvider(values.provider);
      return { name: 'gallery', gallery: { action: 'get', code, dir, provider } };
    }
  }
};

export function parseCommand(argv: string[]): Command {
  const [first, ...rest] = argv;
  if (first === undefined || first === 'help' || first === '-h' || first === '--help')
    return { name: 'help', topic: first === 'help' ? rest[0] : undefined };
  if (first === '-V' || first === '--version' || first === 'version') return { name: 'version' };
  const name = ALIASES[first] ?? first;

  switch (name) {
    case 'chat':
      if (rest[0] === '-h' || rest[0] === '--help') return { name: 'help', topic: 'chat' };
      return { name: 'chat', args: rest };
    case 'gpu':
      throw new UsageError(
        'interactive GPU sessions are now aicentre shell, marimo or code with --gpu'
      );
    case 'self-install':
      parse(rest, {}, false);
      return { name: 'self-install' };
  }

  const spec = Object.hasOwn(COMMANDS, name) ? COMMANDS[name] : undefined;
  if (!spec) throw new UsageError(`unknown command "${first}"`);
  const { values, positionals } = parse(rest, spec.options, spec.positionals);
  if (values.help) return { name: 'help', topic: name };
  return spec.build(values, positionals, name);
}
