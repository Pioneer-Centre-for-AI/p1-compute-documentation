import { statSync, type Stats } from 'node:fs';

export function statOrNull(path: string): Stats | null {
  try {
    return statSync(path);
  } catch {
    return null;
  }
}

export function octal(mode: number): string {
  return (mode & 0o777).toString(8).padStart(4, '0');
}

export function configProblem(st: Stats, uid: number): string | null {
  if (st.uid !== uid && st.uid !== 0) return 'belongs to another user';
  if ((st.mode & 0o022) !== 0) return `others can write to it (mode ${octal(st.mode)})`;
  return null;
}

export function privateKeyProblem(st: Stats, uid: number): string | null {
  if (st.uid === uid && (st.mode & 0o077) !== 0)
    return `mode ${octal(st.mode)} is too open, so ssh ignores it`;
  return null;
}

export function sshDirProblem(st: Stats, uid: number): string | null {
  if (!st.isDirectory()) return 'is not a directory';
  if (st.uid !== uid) return 'belongs to another user';
  if ((st.mode & 0o077) !== 0) return `mode ${octal(st.mode)}; 0700 keeps it private to you`;
  return null;
}
