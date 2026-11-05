import { Type } from 'typebox';
import type { ExtensionAPI, ExtensionFactory } from '@earendil-works/pi-coding-agent';
import type { Ctx, Output } from '../context';
import { cancel } from '../batch/cancel';
import { logs } from '../batch/logs';
import { submit } from '../batch/submit';
import { CLUSTER_IDS, SITE, type ClusterId } from '../check/clusters';
import { check } from '../check/command';
import { listJobs } from '../jobs/list';

export const P1_PROMPT = `You are the P1 compute assistant in aicentre chat. You help researchers at the Pioneer Centre for AI use P1 compute, which is three providers:

- P1 DTU HPC (dtu): LSF. Queue p1 runs batch GPU jobs on seven H100 nodes; p1i is the one queue for interactive GPU work. Project storage is /dtu/p1/<username>; home directories hold 30 GB. login9 takes SSH from the internet; off DTU's network it asks for the DTU password after the key, and on DTU's network or the DTU VPN the key is enough.
- P1 Gefion HPC (gefion): Slurm, reached through Teleport (tsh login). GPU work runs as batch jobs only, charged to P1's Slurm account iu_0096 unless a project has its own.
- P1 NGC HPC (ngc): for sensitive data, air-gapped, reached only through its remote desktop.

How to help:
- Answer from the documentation with p1_docs rather than from memory, and say which page you used. If the docs do not say, say so and suggest the compute coordinator.
- Look at the user's situation with p1_jobs, p1_check and p1_logs before giving advice about it.
- Write job scripts as files in the working directory: #BSUB lines for DTU, #SBATCH lines for Gefion. Then offer p1_submit, which asks the user before it sends anything. Never say a job was submitted or cancelled unless the tool said so.
- Interactive sessions are started by the user in another terminal: aicentre shell, aicentre marimo [notebook] or aicentre code [folder], with --gpu for a GPU on p1i. Suggest the exact command.
- If a tool says a provider wants a password, tell the user to run any aicentre command in another terminal, such as aicentre list jobs, which logs in once for 30 minutes, and then try again.
- Be brief and concrete. Use the user's own paths and job ids.`;

async function command(ctx: Ctx, fn: (c: Ctx) => Promise<number>) {
  let text = '';
  const sink: Output = { write: (s: string) => ((text += s), true), isTTY: false, columns: 100 };
  const quiet = {
    ...ctx,
    out: sink,
    err: sink,
    interactive: false,
    env: { ...ctx.env, NO_COLOR: '1' }
  };
  const code = await fn(quiet);
  return result(text.trim(), { code });
}

function result(text: string, details: Record<string, unknown> = {}) {
  return { content: [{ type: 'text' as const, text: text || '(no output)' }], details };
}

async function asks(
  tool: { hasUI: boolean; ui: { confirm(title: string, message: string): Promise<boolean> } },
  title: string,
  question: string
): Promise<boolean> {
  return tool.hasUI && (await tool.ui.confirm(title, question));
}

const JOB_ID = /^\d+(?:_\d+)?$/;

const Providers = Type.Optional(
  Type.Array(Type.Union(CLUSTER_IDS.map((id) => Type.Literal(id))), {
    description: 'Only these providers; leave out for every provider the user is set up for'
  })
);

function providers(list: unknown): ClusterId[] {
  return Array.isArray(list)
    ? (list.filter((p) => (CLUSTER_IDS as readonly string[]).includes(p)) as ClusterId[])
    : [];
}

export async function fetchDocs(path: string, fetcher: typeof fetch = fetch): Promise<string> {
  const clean = path
    .replace(/^https?:\/\/[^/]+/, '')
    .replace(/^\/+|\/+$/g, '')
    .replace(/\.md$/, '');
  const url = clean ? `${SITE}/${clean}.md` : `${SITE}/llms.txt`;
  const res = await fetcher(url, { signal: AbortSignal.timeout(20_000) });
  if (!res.ok) throw new Error(`${url}: HTTP ${res.status}`);
  return `Source: ${url}\n\n${await res.text()}`;
}

export function p1Extension(ctx: Ctx, fetcher: typeof fetch = fetch): ExtensionFactory {
  return (pi: ExtensionAPI) => {
    pi.registerTool({
      name: 'p1_docs',
      label: 'P1 docs',
      description:
        'Read the P1 compute documentation at hpc.aicentre.dk. With no path, returns the index of every page. ' +
        'With a path such as "providers/dtu", "providers/gefion", "providers/ngc", "cli", "forms" or "gallery/int-gpu", returns that page as markdown.',
      parameters: Type.Object({
        path: Type.Optional(
          Type.String({ description: 'Page path, e.g. providers/dtu; empty for the index' })
        )
      }),
      async execute(_id, params) {
        try {
          return result(await fetchDocs(params.path ?? '', fetcher));
        } catch (error) {
          return result(`Could not read the docs: ${(error as Error).message}`);
        }
      }
    });

    pi.registerTool({
      name: 'p1_jobs',
      label: 'P1 jobs',
      description:
        "The user's running, waiting and suspended jobs across P1 compute, as JSON: provider, id, state, queue, name, node, elapsed, time left, and why a job waits.",
      parameters: Type.Object({ providers: Providers }),
      execute: (_id, params) =>
        command(ctx, (c) => listJobs(c, { clusters: providers(params.providers), json: true }))
    });

    pi.registerTool({
      name: 'p1_check',
      label: 'P1 setup check',
      description:
        'Whether this machine is set up for P1 compute: OpenSSH, ~/.ssh/config Host blocks, keys, the Teleport login for Gefion. As JSON, with a hint for every problem. connect also logs in to each provider.',
      parameters: Type.Object({
        providers: Providers,
        connect: Type.Optional(
          Type.Boolean({ description: 'Also reach each provider over the network and log in' })
        )
      }),
      execute: (_id, params) =>
        command(ctx, (c) =>
          check(c, {
            clusters: providers(params.providers),
            connect: Boolean(params.connect),
            json: true
          })
        )
    });

    pi.registerTool({
      name: 'p1_logs',
      label: 'P1 job output',
      description: "The last lines of a job's output file (or its error file), wherever it ran.",
      parameters: Type.Object({
        job: Type.String({ description: 'The job id' }),
        lines: Type.Optional(
          Type.Integer({
            minimum: 1,
            maximum: 2000,
            description: 'How many lines from the end; default 100'
          })
        ),
        err: Type.Optional(Type.Boolean({ description: 'The error file instead of the output' }))
      }),
      async execute(_id, params) {
        if (!JOB_ID.test(params.job)) return result(`"${params.job}" is not a job id.`);
        const req = {
          id: params.job,
          follow: false,
          lines: params.lines ?? 100,
          err: Boolean(params.err)
        };
        return command(ctx, (c) => logs(c, req));
      }
    });

    pi.registerTool({
      name: 'p1_submit',
      label: 'Submit a P1 batch job',
      description:
        'Submit a job script to the batch queue of the provider it is written for (#BSUB: DTU, #SBATCH: Gefion). ' +
        'The script is a file in the working directory, or a path on the cluster. Asks the user before submitting.',
      parameters: Type.Object({
        script: Type.String({ description: 'Path of the job script' }),
        dir: Type.Optional(
          Type.String({
            description: 'Where on the cluster the job runs; default the project directory'
          })
        )
      }),
      async execute(_id, params, _signal, _update, tool) {
        const what = `Submit ${params.script}${params.dir ? ` to run in ${params.dir}` : ''}?`;
        if (!(await asks(tool, 'Submit a batch job', what)))
          return result('The user did not submit the job.');
        return command(ctx, (c) => submit(c, { script: params.script, dir: params.dir }));
      }
    });

    pi.registerTool({
      name: 'p1_cancel',
      label: 'Cancel P1 jobs',
      description:
        'Cancel jobs, batch or interactive, on whichever provider runs each. Asks the user first.',
      parameters: Type.Object({
        jobs: Type.Array(Type.String({ description: 'A job id' }), { minItems: 1 })
      }),
      async execute(_id, params, _signal, _update, tool) {
        const ids = params.jobs.filter((j) => JOB_ID.test(j));
        if (ids.length !== params.jobs.length) return result('Every job must be a numeric job id.');
        const what = `Cancel ${ids.length === 1 ? `job ${ids[0]}` : `jobs ${ids.join(', ')}`}?`;
        if (!(await asks(tool, 'Cancel jobs', what)))
          return result('The user kept the jobs running.');
        return command(ctx, (c) => cancel(c, { ids }));
      }
    });
  };
}
