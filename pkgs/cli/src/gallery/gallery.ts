import type { Ctx } from '../context';
import { CLUSTER_NAMES, SITE, type ClusterId } from '../check/clusters';
import { loginShell, shq } from '../exec';
import { remotePath, withProvider } from '../session/login';
import { fail } from '../ui/say';
import { makeStyle } from '../ui/style';
import { table } from '../ui/table';

export type Example = {
  code: string;
  slug: string;
  title: string;
  summary: string;
  providers: ClusterId[];
  page: string;
  files: string | null;
  commands: string[];
};

export type GalleryRequest =
  { action: 'list' } | { action: 'get'; code: string; dir?: string; provider?: ClusterId };

export async function examples(ctx: Ctx): Promise<Example[]> {
  const site = (ctx.env.AICENTRE_SITE || SITE).replace(/\/+$/, '');
  const url = `${site}/gallery/index.json`;
  const body = url.startsWith('file://')
    ? await Bun.file(new URL(url).pathname).text()
    : await (await fetch(url, { signal: AbortSignal.timeout(20_000) })).text();
  return (JSON.parse(body) as { examples: Example[] }).examples;
}

export async function gallery(ctx: Ctx, req: GalleryRequest): Promise<number> {
  let all: Example[];
  try {
    all = await examples(ctx);
  } catch (error) {
    return fail(ctx, `cannot read the gallery: ${(error as Error).message}`);
  }
  return req.action === 'list' ? list(ctx, all) : get(ctx, all, req);
}

function list(ctx: Ctx, all: Example[]): number {
  const s = makeStyle(ctx.env, ctx.out);
  const rows = all.map((e) => {
    const where = e.providers.map((p) => CLUSTER_NAMES[p].replace(/^P1 | HPC$/g, '')).join(', ');
    return [e.code, `${e.title} ${s.dim(`· ${where}`)}`];
  });
  const lines = table(rows, { gap: '   ', style: (t, _, c) => (c === 0 ? s.cyan(t) : t) });
  ctx.out.write(`${s.bold('Gallery')}  ${s.dim(`${SITE}/gallery/`)}\n\n${lines.join('\n')}\n`);
  ctx.out.write(
    `\n${s.dim('aicentre gallery get <code> puts an example on the cluster, with what to run next.')}\n`
  );
  return 0;
}

async function get(
  ctx: Ctx,
  all: Example[],
  req: Extract<GalleryRequest, { action: 'get' }>
): Promise<number> {
  const s = makeStyle(ctx.env, ctx.out);
  const example = all.find((e) => e.slug === req.code.toLowerCase());
  if (!example)
    return fail(ctx, `no example ${req.code} in the gallery.`, [
      `There are ${all.map((e) => e.slug).join(', ')}.`
    ]);

  const next = () => {
    if (!example.commands.length) return;
    ctx.out.write(
      `\nNext, from this machine:\n${example.commands.map((c) => `  ${s.dim('$')} ${c}\n`).join('')}`
    );
    if (req.dir) ctx.out.write(s.dim(`  These assume examples/; you put it in ${req.dir}.\n`));
  };
  if (!example.files) {
    ctx.out.write(
      `${example.code} has no files to fetch; it is commands only. ${s.dim(example.page)}\n`
    );
    next();
    return 0;
  }

  const runsOn = `${example.code} runs on ${example.providers.map((p) => CLUSTER_NAMES[p]).join(' and ')}`;
  const choice = {
    asked: req.provider,
    among: example.providers,
    outside: `${runsOn}.`,
    none: [
      `${runsOn}, and this machine is not set up for it.`,
      'aicentre check --all shows what each provider needs.'
    ] as [string, string]
  };
  const files = example.files;
  return withProvider(ctx, choice, async (login) => {
    const parent = remotePath(login, req.dir ?? 'examples');
    const url = shq(files);
    const r = await login.master.run(
      loginShell(
        `mkdir -p ${shq(parent)} && { curl -fsSL ${url} || wget -qO- ${url}; } | tar -xz -C ${shq(parent)}`
      ),
      { timeoutMs: 120_000 }
    );
    if (r.code !== 0)
      return fail(
        ctx,
        `cannot fetch ${example.code} on ${login.alias}: ${r.stderr.trim().split('\n').at(-1) ?? `exit ${r.code}`}`
      );
    ctx.out.write(
      `${s.symbol('ok')} ${example.code} is in ${parent}/${example.slug} on ${CLUSTER_NAMES[login.provider]}\n`
    );
    next();
    return 0;
  });
}
