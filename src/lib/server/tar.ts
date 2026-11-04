// A minimal ustar writer, for the gallery's example tarballs. Every entry gets
// the same owner, mode and time, so the same files always give the same
// bytes, as a reproducible site build needs.
import { gzipSync } from 'node:zlib';

const BLOCK = 512;
const encoder = new TextEncoder();

function field(target: Uint8Array, offset: number, length: number, text: string): void {
  target.set(encoder.encode(text).subarray(0, length), offset);
}

function octal(value: number, length: number): string {
  return value.toString(8).padStart(length - 1, '0') + '\0';
}

function header(name: string, size: number, mtime: number, dir = false): Uint8Array {
  const h = new Uint8Array(BLOCK);
  // Names up to 100 bytes go in `name`; longer paths split into `prefix` and `name`.
  let prefix = '';
  let base = name;
  if (encoder.encode(name).length > 100) {
    const cut = name.lastIndexOf('/', 155);
    prefix = name.slice(0, cut);
    base = name.slice(cut + 1);
  }
  field(h, 0, 100, base);
  field(h, 100, 8, octal(dir ? 0o755 : 0o644, 8));
  field(h, 108, 8, octal(0, 8));
  field(h, 116, 8, octal(0, 8));
  field(h, 124, 12, octal(size, 12));
  field(h, 136, 12, octal(mtime, 12));
  field(h, 148, 8, '        ');
  field(h, 156, 1, dir ? '5' : '0');
  field(h, 257, 6, 'ustar\0');
  field(h, 263, 2, '00');
  field(h, 345, 155, prefix);
  const sum = h.reduce((a, b) => a + b, 0);
  field(h, 148, 8, octal(sum, 7) + ' ');
  return h;
}

/** A .tar.gz of `files` (path -> text), each under `root/`. */
export function tarGz(root: string, files: Record<string, string>, mtime = 0): Uint8Array<ArrayBuffer> {
  const parts: Uint8Array[] = [header(`${root}/`, 0, mtime, true)];
  for (const [path, text] of Object.entries(files).sort(([a], [b]) => a.localeCompare(b))) {
    const body = encoder.encode(text);
    parts.push(header(`${root}/${path}`, body.length, mtime));
    parts.push(body);
    const pad = (BLOCK - (body.length % BLOCK)) % BLOCK;
    if (pad) parts.push(new Uint8Array(pad));
  }
  parts.push(new Uint8Array(BLOCK * 2));
  const total = parts.reduce((n, p) => n + p.length, 0);
  const tar = new Uint8Array(total);
  let at = 0;
  for (const p of parts) {
    tar.set(p, at);
    at += p.length;
  }
  // A plain Uint8Array, which Response takes as a body; Node's Buffer type it is not.
  return new Uint8Array(gzipSync(tar, { level: 9 }));
}
