export type Status = 'ok' | 'warn' | 'fail' | 'skip' | 'info';

export type CheckResult = {
  id: string;
  title: string;
  status: Status;
  detail?: string;
  hint?: string[];
};

export type GroupId = 'local' | 'dtu' | 'gefion' | 'ngc';

export type GroupReport = {
  id: GroupId;
  name: string;
  status: Status;
  note?: string;
  checks: CheckResult[];
};

export interface Sink {
  groupStart(id: GroupId, name: string): void;
  stepStart(group: GroupId, id: string, title: string): void;
  stepEnd(group: GroupId, result: CheckResult): void;
  groupEnd(report: GroupReport): void;
}

const RANK: Record<Status, number> = { skip: 0, info: 1, ok: 2, warn: 3, fail: 4 };

export function worst(statuses: Status[]): Status {
  return statuses.reduce<Status>((a, b) => (RANK[b] > RANK[a] ? b : a), 'skip');
}

type Outcome = Omit<CheckResult, 'id' | 'title'>;

export class Group {
  readonly checks: CheckResult[] = [];
  note?: string;
  skipped = false;

  constructor(
    readonly id: GroupId,
    readonly name: string,
    private readonly sink: Sink
  ) {
    sink.groupStart(id, name);
  }

  async step(
    id: string,
    title: string,
    fn: () => Promise<Outcome> | Outcome
  ): Promise<CheckResult> {
    const fullId = this.id === 'local' ? id : `${this.id}.${id}`;
    this.sink.stepStart(this.id, fullId, title);
    let outcome: Outcome;
    try {
      outcome = await fn();
    } catch (error) {
      outcome = { status: 'fail', detail: `the check itself failed: ${(error as Error).message}` };
    }
    const result: CheckResult = { id: fullId, title, ...outcome };
    this.checks.push(result);
    this.sink.stepEnd(this.id, result);
    return result;
  }

  skip(id: string, title: string, detail: string): Promise<CheckResult> {
    return this.step(id, title, () => ({ status: 'skip', detail }));
  }

  end(): GroupReport {
    const report: GroupReport = {
      id: this.id,
      name: this.name,
      status: this.skipped ? 'skip' : worst(this.checks.map((c) => c.status)),
      note: this.note,
      checks: this.checks
    };
    this.sink.groupEnd(report);
    return report;
  }
}
