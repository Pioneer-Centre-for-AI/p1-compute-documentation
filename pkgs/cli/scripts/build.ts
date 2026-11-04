import { cpSync, existsSync, mkdirSync, rmSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { parseArgs } from 'node:util';
import type { Build } from 'bun';

const { values } = parseArgs({
  options: {
    target: { type: 'string' },
    outfile: { type: 'string', default: 'dist/aicentre' },
    version: { type: 'string' }
  }
});

const pkg = (await Bun.file(new URL('../package.json', import.meta.url)).json()) as {
  version: string;
};
const version = values.version ?? pkg.version;

const result = await Bun.build({
  entrypoints: [new URL('../src/main.ts', import.meta.url).pathname],
  compile: {
    ...(values.target ? { target: values.target as Build.CompileTarget } : {}),
    outfile: values.outfile,
    autoloadDotenv: false,
    autoloadBunfig: false
  },
  minify: true,
  sourcemap: 'linked',
  define: { AICENTRE_VERSION: JSON.stringify(version) }
});

if (!result.success) {
  for (const log of result.logs) console.error(log);
  process.exit(1);
}

const pi = new URL('../node_modules/@earendil-works/pi-coding-agent/', import.meta.url).pathname;
const assets = join(dirname(values.outfile!), 'pi');
rmSync(assets, { recursive: true, force: true });
mkdirSync(assets, { recursive: true });
for (const [from, to] of [
  ['package.json', 'package.json'],
  ['README.md', 'README.md'],
  ['CHANGELOG.md', 'CHANGELOG.md'],
  ['docs', 'docs'],
  ['dist/modes/interactive/theme', 'theme'],
  ['dist/modes/interactive/assets', 'assets'],
  ['dist/core/export-html', 'export-html']
] as const) {
  if (existsSync(join(pi, from))) cpSync(join(pi, from), join(assets, to), { recursive: true });
}
console.log(
  `built ${values.outfile} (aicentre ${version}${values.target ? `, ${values.target}` : ''})`
);
