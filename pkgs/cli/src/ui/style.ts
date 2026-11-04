import type { Env, Output } from '../context';
import type { Status } from '../check/types';

export type Style = {
  color: boolean;
  green(s: string): string;
  red(s: string): string;
  yellow(s: string): string;
  cyan(s: string): string;
  dim(s: string): string;
  bold(s: string): string;
  symbol(status: Status): string;
  spinner(frame: number): string;
};

export function colorEnabled(env: Env, out: Output): boolean {
  if (env.NO_COLOR) return false;
  if (env.FORCE_COLOR && env.FORCE_COLOR !== '0') return true;
  if (env.TERM === 'dumb') return false;
  return Boolean(out.isTTY);
}

const SPINNER = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
const SYMBOLS: Record<Status, string> = { ok: '✓', warn: '!', fail: '✗', skip: '○', info: '·' };
const ASCII: Record<Status, string> = { ok: '+', warn: '!', fail: 'x', skip: '-', info: '*' };

export function makeStyle(env: Env, out: Output): Style {
  const color = colorEnabled(env, out);
  const ascii = env.TERM === 'dumb' || env.AICENTRE_ASCII === '1';
  const sgr = (open: number, close: number) => (s: string) =>
    color ? `\x1b[${open}m${s}\x1b[${close}m` : s;
  const green = sgr(32, 39);
  const red = sgr(31, 39);
  const yellow = sgr(33, 39);
  const cyan = sgr(36, 39);
  const dim = sgr(2, 22);
  const bold = sgr(1, 22);
  const paint: Record<Status, (s: string) => string> = {
    ok: green,
    warn: yellow,
    fail: red,
    skip: dim,
    info: cyan
  };
  return {
    color,
    green,
    red,
    yellow,
    cyan,
    dim,
    bold,
    symbol: (status) => paint[status]((ascii ? ASCII : SYMBOLS)[status]),
    spinner: (frame) => cyan(ascii ? '-\\|/'[frame % 4]! : SPINNER[frame % SPINNER.length]!)
  };
}
