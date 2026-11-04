import type { Env, Output } from '../context';
import { LiveArea } from './live';
import { makeStyle, type Style } from './style';

export class Steps {
  readonly style: Style;
  private readonly live: LiveArea | undefined;
  private current: { text: string; since: number } | undefined;

  constructor(
    env: Env,
    private readonly out: Output
  ) {
    this.style = makeStyle(env, out);
    const tty = Boolean(out.isTTY) && env.TERM !== 'dumb';
    this.live = tty
      ? new LiveArea(out, (frame) => {
          if (!this.current) return [];
          const secs = Math.floor((Date.now() - this.current.since) / 1000);
          const took =
            secs >= 3
              ? this.style.dim(`  ${secs < 120 ? `${secs} s` : `${Math.floor(secs / 60)} min`}`)
              : '';
          return [`${this.style.spinner(frame)} ${this.current.text}${took}`];
        })
      : undefined;
    this.live?.start();
  }

  private print(lines: string[]): void {
    if (this.live) this.live.print(lines);
    else this.out.write(lines.map((l) => l + '\n').join(''));
  }

  step(text: string): void {
    this.finish();
    this.current = { text, since: Date.now() };
    if (!this.live) this.out.write(`${this.style.symbol('info')} ${text}\n`);
  }

  note(text: string): void {
    this.print([`  ${this.style.dim(text)}`]);
  }

  private finish(): void {
    if (this.current && this.live) this.print([`${this.style.symbol('ok')} ${this.current.text}`]);
    this.current = undefined;
  }

  fail(text: string, hint: string[] = []): void {
    this.current = undefined;
    this.print([
      `${this.style.symbol('fail')} ${text}`,
      ...hint.map((h) =>
        h.startsWith('  ') ? `    ${this.style.cyan(h)}` : `    ${this.style.dim(h)}`
      )
    ]);
  }

  done(): void {
    this.finish();
    this.live?.stop();
  }
}
