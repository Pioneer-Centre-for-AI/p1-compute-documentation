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

export type RfcStatus = (typeof STATUSES)[number];

export const RfcFrontmatter = z
  .object({
    title: z.string(),
    description: z.string(),
    authors: z.array(z.string()).min(1),
    created: day,
    updated: day,
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
  });

export type RfcMeta = z.infer<typeof RfcFrontmatter>;
export type RfcEntry = { number: string; href: string; meta: RfcMeta };

export const STATUS_LABELS: Record<RfcStatus, string> = {
  open: 'Open',
  accepted: 'Accepted',
  rejected: 'Rejected',
  deferred: 'Deferred',
  withdrawn: 'Withdrawn'
};

const modules = import.meta.glob(['/src/routes/rfc/*/+page.svx', '!/src/routes/rfc/all/+page.svx'], {
  eager: true
}) as Record<string, { metadata?: unknown }>;

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
