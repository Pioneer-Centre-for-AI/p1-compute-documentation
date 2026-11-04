import { mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import type { Paths } from '../paths';

export type SessionNote = {
  jobid: string;
  app: string;
  pid: number;
  startedAt: string;
  url?: string;
};

function dir(paths: Paths): string {
  return join(paths.state, 'gpu');
}

export function saveNote(paths: Paths, note: SessionNote): void {
  mkdirSync(dir(paths), { recursive: true });
  writeFileSync(join(dir(paths), `${note.jobid}.json`), JSON.stringify(note, null, 2) + '\n');
}

export function dropNote(paths: Paths, jobid: string): void {
  rmSync(join(dir(paths), `${jobid}.json`), { force: true });
}

function alive(pid: number): boolean {
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

export function liveNotes(paths: Paths): Map<string, SessionNote> {
  const notes = new Map<string, SessionNote>();
  let names: string[] = [];
  try {
    names = readdirSync(dir(paths));
  } catch {
    return notes;
  }
  for (const name of names) {
    try {
      const note = JSON.parse(readFileSync(join(dir(paths), name), 'utf8')) as SessionNote;
      if (alive(note.pid)) notes.set(note.jobid, note);
      else rmSync(join(dir(paths), name), { force: true });
    } catch {
      rmSync(join(dir(paths), name), { force: true });
    }
  }
  return notes;
}
