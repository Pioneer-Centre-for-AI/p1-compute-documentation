export type Scheduler = 'lsf' | 'slurm' | 'pbs';

const PREFIX: Record<Scheduler, string> = { lsf: '#BSUB', slurm: '#SBATCH', pbs: '#PBS' };

export const PROVIDER_OF: Record<Scheduler, 'dtu' | 'gefion' | 'ngc'> = {
  lsf: 'dtu',
  slurm: 'gefion',
  pbs: 'ngc'
};

export function schedulerOf(text: string): Scheduler | null {
  for (const line of text.split('\n')) {
    const t = line.trim();
    for (const [scheduler, prefix] of Object.entries(PREFIX) as [Scheduler, string][]) {
      if (t.startsWith(`${prefix} `) || t === prefix) return scheduler;
    }
  }
  return null;
}

export function directiveWords(text: string, scheduler: Scheduler): string[] {
  const prefix = PREFIX[scheduler];
  return text
    .split('\n')
    .map((l) => l.trim())
    .filter((l) => l.startsWith(`${prefix} `))
    .flatMap((l) =>
      l
        .slice(prefix.length)
        .trim()
        .replace(/\s+#.*$/, '')
        .split(/\s+/)
    );
}

export function sets(words: string[], options: string[]): boolean {
  return words.some((w) =>
    options.some((o) => w === o || (o.startsWith('--') && w.startsWith(`${o}=`)))
  );
}
