import type { Output } from '../context';
import type { CheckResult, GroupId, GroupReport, Sink, Status } from '../check/types';
import { LiveArea } from './live';
import type { Style } from './style';

type GroupState = {
  id: GroupId;
  name: string;
  checks: CheckResult[];
  running?: string;
  report?: GroupReport;
  printed: boolean;
};

export type Summary = { ok: boolean; lines: string[] };

export interface Reporter extends Sink {
  finish(reports: GroupReport[], summary: Summary): void;
  abort(): void;
}

function checkLines(c: CheckResult, s: Style): string[] {
  const loud = c.status === 'fail' || c.status === 'warn';
  const detail = c.detail ? `  ${loud ? c.detail : s.dim(c.detail)}` : '';
  const lines = [`  ${s.symbol(c.status)} ${c.title}${detail}`];
  if (loud)
    for (const h of c.hint ?? [])
      lines.push(h.startsWith('  ') ? `    ${s.cyan(h)}` : h ? `    ${s.dim(h)}` : '');
  return lines;
}

function groupLines(g: GroupState, s: Style, frame?: number): string[] {
  if (g.report?.status === 'skip' && g.report.note) {
    return [`${s.symbol('skip')} ${s.bold(g.name)}  ${s.dim(g.report.note)}`, ''];
  }
  const lines = [s.bold(g.name)];
  for (const c of g.checks) lines.push(...checkLines(c, s));
  if (g.running !== undefined && frame !== undefined)
    lines.push(`  ${s.spinner(frame)} ${g.running}`);
  lines.push('');
  return lines;
}

abstract class Ordered implements Reporter {
  protected groups: GroupState[] = [];

  constructor(protected readonly style: Style) {}

  groupStart(id: GroupId, name: string): void {
    this.groups.push({ id, name, checks: [], printed: false });
  }
  stepStart(group: GroupId, _id: string, title: string): void {
    this.find(group).running = title;
  }
  stepEnd(group: GroupId, result: CheckResult): void {
    const g = this.find(group);
    g.running = undefined;
    g.checks.push(result);
  }
  groupEnd(report: GroupReport): void {
    this.find(report.id).report = report;
    this.flush();
  }

  protected flush(): void {
    for (const g of this.groups) {
      if (g.printed) continue;
      if (!g.report) return;
      g.printed = true;
      this.emit(groupLines(g, this.style));
    }
  }

  protected abstract emit(lines: string[]): void;
  abstract finish(reports: GroupReport[], summary: Summary): void;
  abort(): void {}

  private find(id: GroupId): GroupState {
    const g = this.groups.find((x) => x.id === id);
    if (!g) throw new Error(`no group ${id}`);
    return g;
  }
}

export class LiveReporter extends Ordered {
  private readonly area: LiveArea;

  constructor(
    style: Style,
    private readonly out: Output
  ) {
    super(style);
    this.area = new LiveArea(out, (frame) =>
      this.groups.filter((g) => !g.printed).flatMap((g) => groupLines(g, this.style, frame))
    );
    this.area.start();
  }

  protected emit(lines: string[]): void {
    this.area.print(lines);
  }

  finish(_reports: GroupReport[], summary: Summary): void {
    this.area.stop();
    this.out.write(summary.lines.map((l) => l + '\n').join(''));
  }

  abort(): void {
    this.area.stop();
  }
}

export class PlainReporter extends Ordered {
  constructor(
    style: Style,
    private readonly out: Output
  ) {
    super(style);
  }

  protected emit(lines: string[]): void {
    this.out.write(lines.map((l) => l + '\n').join(''));
  }

  finish(_reports: GroupReport[], summary: Summary): void {
    this.emit(summary.lines);
  }
}

export class JsonReporter implements Reporter {
  constructor(
    private readonly out: Output,
    private readonly meta: { version: string; connect: boolean }
  ) {}
  groupStart(): void {}
  stepStart(): void {}
  stepEnd(): void {}
  groupEnd(): void {}
  abort(): void {}
  finish(reports: GroupReport[], summary: Summary): void {
    const doc = {
      version: this.meta.version,
      connect: this.meta.connect,
      ok: summary.ok,
      groups: reports
    };
    this.out.write(JSON.stringify(doc, null, 2) + '\n');
  }
}

export function summarize(reports: GroupReport[], style: Style, connect: boolean): Summary {
  const count: Record<Status, number> = { ok: 0, warn: 0, fail: 0, skip: 0, info: 0 };
  for (const r of reports) for (const c of r.checks) count[c.status]++;
  const checked = reports.filter((r) => r.id !== 'local' && r.status !== 'skip');
  const ok = count.fail === 0;

  const parts = [style.green(`${count.ok} passed`)];
  if (count.warn)
    parts.push(style.yellow(`${count.warn} ${count.warn === 1 ? 'warning' : 'warnings'}`));
  if (count.fail) parts.push(style.red(`${count.fail} failed`));
  const lines = [parts.join(style.dim(' · '))];

  if (checked.length === 0) {
    lines.push(
      style.dim(
        'No P1 provider is set up on this machine yet. aicentre check --all shows what each one needs.'
      )
    );
  } else if (ok && !connect) {
    lines.push(
      style.dim('Files look right. aicentre check --connect also logs in to each provider.')
    );
  }
  return { ok, lines };
}
