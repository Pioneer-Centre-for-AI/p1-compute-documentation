# P1's cluster documentation

Documentation site for the P1 compute clusters, built with [SvelteKit](https://kit.svelte.dev/) and statically deployed to GitHub Pages at [hpc.aicentre.dk](https://hpc.aicentre.dk).

## Local development

The project uses [`bun`](https://bun.sh/) as its package manager and provides a `nix` flake for reproducible tooling.

### With nix (recommended)

```bash
git clone git@github.com:Pioneer-Centre-for-AI/p1-compute-documentation.git
cd p1-compute-documentation

# Enter the dev shell (bun + node)
nix develop

bun install
bun run dev
```

With `direnv` installed, `cd` into the repo and run `direnv allow` once — the dev shell loads automatically.

### Without nix

Install `bun` (≥ 1.1) however you prefer (`brew install bun`, the official installer, etc.), then:

```bash
bun install
bun run dev
```

The site is served at `http://localhost:5173/`.

## Useful scripts

- `bun run dev` — Vite dev server
- `bun run build` — static build to `build/`
- `bun run preview` — serve the built site
- `bun run check` — `svelte-check` type/diagnostics

## Reproducible build with nix

```bash
# Build the static site into ./result (a symlink into the nix store)
nix build .#website

# Build and serve it on http://127.0.0.1:8080 (override with PORT=...)
nix run .#website
```

If `package.json` or `bun.lock` changes, `nix build` will fail with a hash
mismatch on the `node_modules` fixed-output derivation — copy the printed
`got:` hash into `nodeModulesHash` in `flake.nix` and rebuild.

That hash is recorded **per system**, because bun resolves platform-specific
native packages (esbuild, rollup, lightningcss, tailwind oxide). Updating it on
a mac fixes local builds but not CI, which runs `x86_64-linux`; take the `got:`
hash from the failed CI log and update that entry too.

Nix builds from the git tree, so **a new file that has not been `git add`ed is
invisible to `nix build`**. The symptom is a `vite:load-fallback` ENOENT for a
file that plainly exists on disk. Stage it and rebuild.

```bash
# Type check and site build, the same two gates CI runs
nix flake check
```
