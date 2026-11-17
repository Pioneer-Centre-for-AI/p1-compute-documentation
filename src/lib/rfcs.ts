import { z } from 'zod';

const day = z.preprocess(
  (value) => {
    if (value instanceof Date) return value.toISOString().slice(0, 10);
    if (typeof value === 'string' && /^\d{4}-\d{2}-\d{2}T00:00:00(\.000)?Z$/.test(value)) {
      return value.slice(0, 10);
    }
    return value;
  },
  z.string().regex(/^\d{4}-\d{2}-\d{2}$/, 'Expected a date as YYYY-MM-DD')
);

const STATUSES = ['open', 'accepted', 'rejected', 'deferred', 'withdrawn'] as const;

const version = z.string().regex(/^\d+\.\d+$/, 'Write the version as a quoted string, such as "0.1"');

export type RfcStatus = (typeof STATUSES)[number];

const documentFields = {
  title: z.string(),
  description: z.string(),
  authors: z.array(z.string()).min(1),
  created: day,
  updated: day,
  version,
  changes: z.array(z.object({ version, date: day, note: z.string() })).min(1)
};

type Versioned = { version: string; updated: string; changes: { version: string; date: string }[] };

function checkChanges(meta: Versioned, ctx: z.RefinementCtx) {
  const [newest] = meta.changes;
  if (newest.version !== meta.version || newest.date !== meta.updated) {
    ctx.addIssue({
      code: 'custom',
      path: ['changes', 0],
      message: 'The first entry in changes must have the same version and date as version and updated.'
    });
  }
}

export const RfcFrontmatter = z
  .object({
    ...documentFields,
    status: z.enum(STATUSES),
    decided: day.optional(),
    summary: z.string().optional()
  })
  .superRefine((meta, ctx) => {
    if (meta.status !== 'open' && !meta.decided) {
      ctx.addIssue({ code: 'custom', path: ['decided'], message: 'A decided RFC needs a decided date.' });
    }
    if (meta.status === 'open' && meta.decided) {
      ctx.addIssue({ code: 'custom', path: ['decided'], message: 'An open RFC has no decided date.' });
    }
    checkChanges(meta, ctx);
    if (meta.status === 'accepted' && Number(meta.version.split('.')[0]) < 1) {
      ctx.addIssue({ code: 'custom', path: ['version'], message: 'An accepted RFC has version 1.0 or higher.' });
    }
  });

export const ConventionsFrontmatter = z.object(documentFields).superRefine(checkChanges);

export type ConventionsMeta = z.infer<typeof ConventionsFrontmatter>;

export type RfcMeta = z.infer<typeof RfcFrontmatter>;
export type RfcEntry = { number: string; href: string; meta: RfcMeta };

export const STATUS_LABELS: Record<RfcStatus, string> = {
  open: 'Open',
  accepted: 'Accepted',
  rejected: 'Rejected',
  deferred: 'Deferred',
  withdrawn: 'Withdrawn'
};

const modules = import.meta.glob(['/src/routes/rfc/*/+page.svx', '!/src/routes/rfc/all/+page.svx', '!/src/routes/rfc/conventions/+page.svx'], {
  eager: true
}) as Record<string, { metadata?: unknown }>;

export const CONVENTIONS_HREF = '/rfc/conventions/';

const conventionsModule = import.meta.glob('/src/routes/rfc/conventions/+page.svx', {
  eager: true
}) as Record<string, { metadata?: unknown }>;

var conventions: ConventionsMeta | undefined;

export function getConventions(): ConventionsMeta {
  conventions ??= ConventionsFrontmatter.parse(Object.values(conventionsModule)[0]?.metadata);
  return conventions;
}

var parsed: RfcEntry[] | null | undefined;

export function getRfcs(): RfcEntry[] {
  parsed ??= Object.entries(modules)
    .map(([path, mod]) => {
      const number = path.split('/').at(-2)!;
      if (!/^\d{4}$/.test(number)) {
        throw new Error(`RFC folder "${number}" in ${path} must be a four-digit number, such as 0001.`);
      }
      return { number, href: `/rfc/${number}/`, meta: RfcFrontmatter.parse(mod.metadata) };
    })
    .sort((a, b) => a.number.localeCompare(b.number));
  return parsed;
}

export function openRfcs(limit: number): RfcEntry[] {
  return getRfcs()
    .filter((r) => r.meta.status === 'open')
    .toSorted((a, b) => b.meta.updated.localeCompare(a.meta.updated) || b.number.localeCompare(a.number))
    .slice(0, limit);
}

export function rfcsNewestFirst(): RfcEntry[] {
  return getRfcs().toReversed();
}

export function rfcTitle(rfc: RfcEntry): string {
  return `RFC ${rfc.number}: ${rfc.meta.title}`;
}

export function formatDay(value: string, month: 'long' | 'short' = 'long'): string {
  return new Date(`${value}T00:00:00Z`).toLocaleDateString('en-GB', {
    day: 'numeric',
    month,
    year: 'numeric',
    timeZone: 'UTC'
  });
}
