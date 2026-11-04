import type { Style } from '../ui/style';

type Row = [name: string, description: string];
type Section = { title: string; rows?: Row[]; lines?: string[] };
type Page = { summary: string; usage: string[]; sections: Section[]; note?: string };

const DOCS = 'https://hpc.aicentre.dk/cli/';

const PROVIDER: Row = ['-p, --provider <name>', 'Only these providers: dtu, gefion, ngc'];
const JSON_ROW: Row = ['--json', 'Print JSON'];

const SESSION_OPTIONS: Row[] = [
  ['--gpu', 'On a GPU node instead of the login node'],
  ['--time <2h>', 'With --gpu: run limit, as 90, 2h or 1:30 (default 1h)'],
  ['--cpus <n>', 'With --gpu: CPU cores (default 4)'],
  ['--mem <8GB>', 'With --gpu: memory per core (default 8GB)'],
  ['-p, --provider <name>', 'Another provider than the first you use']
];

const SESSION_NOTE =
  'Without --gpu it runs on the login node: for editing, installs and light runs. ' +
  "With --gpu it is a job with one GPU on p1i, P1 DTU HPC's interactive queue. " +
  'Paths are on the cluster, relative to your project directory. Ctrl-C ends the session.';

function sessionPage(app: 'shell' | 'marimo' | 'code'): Page {
  const extra: Row[] =
    app === 'code'
      ? [
          ['--server openvscode', 'The open build of the server, with Open VSX'],
          ['--no-browser', 'Print the address without opening it']
        ]
      : app === 'marimo'
        ? [['--no-browser', 'Print the address without opening it']]
        : [];
  return {
    summary:
      app === 'shell'
        ? 'A shell on P1 compute, in this terminal'
        : app === 'marimo'
          ? 'marimo on P1 compute, in your browser'
          : 'VS Code on P1 compute, in your browser',
    usage: [`aicentre ${app} [${app === 'marimo' ? 'notebook|folder' : 'folder'}] [options]`],
    sections: [{ title: 'Options', rows: [...SESSION_OPTIONS, ...extra] }],
    note: SESSION_NOTE
  };
}

const PAGES: Record<string, Page> = {
  main: {
    summary: 'Work with P1 compute from your laptop',
    usage: ['aicentre <command> [options]'],
    sections: [
      {
        title: 'Interactive',
        rows: [
          ['shell', 'A shell on a login node, or a GPU node with --gpu'],
          ['marimo', 'marimo in your browser, with or without a GPU'],
          ['code', 'VS Code in your browser, with or without a GPU']
        ]
      },
      {
        title: 'Batch',
        rows: [
          ['submit', 'Send a job script to the batch queue'],
          ['list jobs', 'Your jobs across every provider'],
          ['logs', "A job's output; -f follows it"],
          ['cancel', 'End jobs']
        ]
      },
      {
        title: 'Data',
        rows: [
          ['push', 'Copy code or data to the cluster'],
          ['pull', 'Copy results back from the cluster'],
          ['gallery', "The site's examples; gallery get puts one on the cluster"]
        ]
      },
      {
        title: 'Assistant',
        rows: [['chat', 'Ask an agent about P1 compute, your jobs and setup']]
      },
      {
        title: 'Setup',
        rows: [
          ['check', 'Check that this machine is ready for P1 compute'],
          ['upgrade', 'Update to the latest release'],
          ['uninstall', 'Remove aicentre']
        ]
      },
      {
        title: 'Examples',
        lines: [
          'aicentre marimo project/notebook.py',
          'aicentre code project --gpu --time 2h',
          'aicentre submit train.sh && aicentre list jobs'
        ]
      }
    ],
    note: 'Work in progress: illustrative for now, and it may change. It covers every provider you use; add --help to any command for more.'
  },

  shell: sessionPage('shell'),
  marimo: sessionPage('marimo'),
  code: sessionPage('code'),

  submit: {
    summary: 'Send a job script to the batch queue',
    usage: ['aicentre submit <script> [options]'],
    sections: [
      {
        title: 'Options',
        rows: [
          ['--dir <path>', 'Where the job runs (default: your project directory)'],
          ['-p, --provider <name>', 'For a script without #BSUB or #SBATCH lines']
        ]
      }
    ],
    note:
      'The script picks the provider: #BSUB lines go to P1 DTU HPC, #SBATCH lines to P1 Gefion HPC. ' +
      'A script on this machine is copied up first; any other path is read on the cluster.'
  },

  logs: {
    summary: "A job's output",
    usage: ['aicentre logs <job> [options]'],
    sections: [
      {
        title: 'Options',
        rows: [
          ['-f, --follow', 'Keep printing as the job writes'],
          ['-n, --lines <n>', 'How many lines from the end (default 50)'],
          ['--err', 'The error file instead of the output'],
          ['-p, --provider <name>', 'For a job not submitted from this machine']
        ]
      }
    ]
  },

  cancel: {
    summary: 'End jobs, batch or interactive',
    usage: ['aicentre cancel <job...> [options]'],
    sections: [
      {
        title: 'Options',
        rows: [['-p, --provider <name>', 'For jobs not submitted from this machine']]
      }
    ]
  },

  push: {
    summary: 'Copy code or data to the cluster',
    usage: ['aicentre push <local> [remote] [options]'],
    sections: [
      {
        title: 'Options',
        rows: [['-p, --provider <name>', 'Another provider than the first you use']]
      }
    ],
    note: 'Like cp -r: datasets/ lands in <remote>/datasets. Remote paths are relative to your project directory. It resumes where a stopped copy left off.'
  },

  pull: {
    summary: 'Copy results back from the cluster',
    usage: ['aicentre pull <remote> [local] [options]'],
    sections: [
      {
        title: 'Options',
        rows: [['-p, --provider <name>', 'Another provider than the first you use']]
      }
    ],
    note: 'The remote path is relative to your project directory; local defaults to this folder.'
  },

  gallery: {
    summary: 'The examples at hpc.aicentre.dk/gallery',
    usage: ['aicentre gallery', 'aicentre gallery get <code> [dir] [options]'],
    sections: [
      {
        title: 'Options',
        rows: [['-p, --provider <name>', 'Another provider than the first you use']]
      }
    ],
    note: 'get has the login node download the example itself, into examples/<code> in your project directory, and prints what to run next. Nothing to clone.'
  },

  chat: {
    summary: 'An agent that knows P1 compute and can see your jobs',
    usage: ['aicentre chat [options] [message]'],
    sections: [
      {
        title: 'It can',
        rows: [
          ['read', 'The P1 docs, your jobs, your setup and job output'],
          ['write', 'Job scripts and other drafts in this folder'],
          ['submit, cancel', 'Only after you say yes in the chat']
        ]
      },
      {
        title: 'Options',
        rows: [
          ['--model <provider/id>', 'The model to talk to'],
          ['-c, --continue', 'Pick up the last conversation'],
          ['/login', 'Inside the chat: sign in to a model provider']
        ]
      }
    ],
    note: 'An agent with P1 tools. The model is yours to choose: sign in with /login or set a provider key, such as ANTHROPIC_API_KEY. Settings live in ~/.config/aicentre/agent, conversations in ~/.local/state/aicentre/chat.'
  },

  check: {
    summary: 'Check that this machine is ready for P1 compute',
    usage: ['aicentre check [options]'],
    sections: [
      {
        title: 'Options',
        rows: [
          ['--connect', 'Also log in: tests the VPN, your key and your sign-in'],
          ['--all', 'Include providers not set up here, with what they need'],
          PROVIDER,
          JSON_ROW
        ]
      }
    ],
    note: 'Looks at ~/.ssh, your keys and your Gefion sign-in. Exits 1 when a check fails.'
  },

  list: {
    summary: 'Your running and waiting jobs across P1 compute',
    usage: ['aicentre list jobs [options]'],
    sections: [{ title: 'Options', rows: [PROVIDER, JSON_ROW] }],
    note: 'Asks every provider you use at once. NGC is air-gapped: use qstat on its remote desktop.'
  },

  upgrade: {
    summary: 'Update to the latest release',
    usage: ['aicentre upgrade [options]'],
    sections: [{ title: 'Options', rows: [['--version <v>', 'Install this version instead']] }],
    note: 'Checks the download against its checksum, then deletes the version it replaces.'
  },

  uninstall: {
    summary: 'Remove aicentre from this machine',
    usage: ['aicentre uninstall [options]'],
    sections: [
      { title: 'Options', rows: [['--purge', 'Also remove your settings, without asking']] }
    ],
    note: 'Removes the program and its state, cache and runtime files. Asks before removing ~/.config/aicentre.'
  }
};

function wrap(text: string, width = 78): string[] {
  const lines: string[] = [];
  let line = '';
  for (const word of text.split(' ')) {
    if (line && line.length + 1 + word.length > width) {
      lines.push(line);
      line = word;
    } else line = line ? `${line} ${word}` : word;
  }
  if (line) lines.push(line);
  return lines;
}

export function renderHelp(topic: string | undefined, style: Style, version: string): string {
  const known = topic !== undefined && topic in PAGES && topic !== 'main';
  const page = known ? PAGES[topic]! : PAGES.main!;
  const out: string[] = [];

  const title = known ? `aicentre ${topic === 'list' ? 'list jobs' : topic}` : 'aicentre';
  out.push(
    `${style.bold(title)}${known ? '' : ` ${style.dim(version)}`}  ${style.dim(page.summary)}`,
    ''
  );
  page.usage.forEach((u, i) => out.push(`${i === 0 ? style.bold('Usage') : '     '}  ${u}`));
  const width = Math.max(
    0,
    ...page.sections.flatMap((section) => (section.rows ?? []).map(([name]) => name.length))
  );
  for (const section of page.sections) {
    out.push('', style.bold(section.title));
    for (const [name, description] of section.rows ?? [])
      out.push(`  ${style.cyan(name.padEnd(width))}   ${description}`);
    for (const line of section.lines ?? []) out.push(`  ${style.dim('$')} ${line}`);
  }
  out.push('');
  if (page.note) out.push(...wrap(page.note).map((l) => style.dim(l)));
  out.push(style.dim(`Docs: ${DOCS}`));
  return out.join('\n') + '\n';
}
