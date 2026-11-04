import type { Output } from '../context';

const ANSI = /\x1b\[[0-9;?]*[A-Za-z]/y;
const segmenter = new Intl.Segmenter();

export function truncate(line: string, max: number): string {
  if (Bun.stringWidth(line) <= max) return line;
  let out = '';
  let width = 0;
  let i = 0;
  while (i < line.length) {
    ANSI.lastIndex = i;
    const esc = ANSI.exec(line);
    if (esc) {
      out += esc[0];
      i += esc[0].length;
      continue;
    }
    const next = line.indexOf('\x1b', i);
    const text = line.slice(i, next === -1 ? undefined : next);
    for (const { segment } of segmenter.segment(text)) {
      const w = Bun.stringWidth(segment);
      if (width + w > max - 1) return `${out}…\x1b[0m`;
      out += segment;
      width += w;
    }
    i += text.length;
  }
  return out;
}

export class LiveArea {
  private height = 0;
  private frame = 0;
  private timer: ReturnType<typeof setInterval> | undefined;

  constructor(
    private readonly out: Output & { rows?: number },
    private readonly render: (frame: number) => string[],
    private readonly intervalMs = 80
  ) {}

  start(): void {
    this.out.write('\x1b[?25l');
    this.draw();
    this.timer = setInterval(() => this.draw(), this.intervalMs);
  }

  print(lines: string[]): void {
    this.out.write(`\x1b[?2026h${this.clear()}${lines.map((l) => l + '\n').join('')}\x1b[?2026l`);
    this.height = 0;
    this.draw();
  }

  draw(): void {
    const cols = Math.max(20, this.out.columns ?? 80);
    const rows = Math.max(5, this.out.rows ?? 24);
    let lines = this.render(this.frame++).map((l) => truncate(l, cols - 1));
    if (lines.length > rows - 1) lines = lines.slice(-(rows - 1));
    this.out.write(`\x1b[?2026h${this.clear()}${lines.map((l) => l + '\n').join('')}\x1b[?2026l`);
    this.height = lines.length;
  }

  stop(): void {
    if (this.timer) clearInterval(this.timer);
    this.timer = undefined;
    this.out.write(`${this.clear()}\x1b[?25h`);
    this.height = 0;
  }

  private clear(): string {
    return this.height ? `\r\x1b[${this.height}A\x1b[J` : '\r\x1b[J';
  }
}
