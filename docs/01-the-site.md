# The site

The site at <https://hpc.aicentre.dk> is SvelteKit with `adapter-static`,
built to `build/` and served by GitHub Pages. Pages are mdsvex `.svx` files
under `src/routes/`. bun is the package manager.

## Develop

```sh
nix develop          # bun, node and shellcheck; or `direnv allow` once
bun install
bun run dev          # http://localhost:5173/
bun run check        # svelte-check
bun run build        # static build to build/
bun run preview      # serve the built site
```

Without Nix, any bun from 1.1 works for the site.

## Collections

Two kinds of page carry typed YAML frontmatter, validated with zod at build
time. A bad page fails the build.

- Provider pages, `src/routes/providers/<slug>/+page.svx`, through
  `src/lib/content.ts`. Every provider page opens by pointing at the
  provider's own documentation (`ProviderDocs`): this site covers overview and
  onboarding, and the provider owns the deep reference. The quota keys
  (`wallTime`, `gpusPerJob`, `gpusAtOnce`, `homeDirectory`, `projectStorage`,
  `sharedPool`) are all required. A cluster that does not publish a limit has
  to say so in words. The labels live in `ClusterQuotas.svelte`, so the three
  pages read the same.
- Gallery pages, through `src/lib/gallery.ts`. See
  [11 · The gallery](11-the-gallery.md).

## Markdown twins and llms.txt

Every page has a raw markdown twin at `<path>.md` (`/about/` gives
`/about.md`). `src/lib/markdown/twins.ts` renders them from the `.svx`
sources, and three endpoints share it: `[...path].md` serves one twin,
`llms-full.txt` joins all of them, and `llms.txt` lists them per llmstxt.org.
The glob in `twins.ts` pulls every page source into the bundle, so it is
server-only.

`MARKDOWN_PAGES` in `src/lib/markdownPages.ts` lists the twin paths by hand.
The root layout imports it in the browser, and a glob there would ship every
page's source to the client. The `.md` endpoint asserts that the list matches
the `.svx` pages, so a missing entry fails the build.

`llms.txt` follows `src/lib/nav.ts`. The gallery is not in the nav, so it is
added by hand. Pages kept out of the nav (Terms, Privacy) go under
`## Optional`.

## Endpoints

All prerendered, so they become files in `build/`.

| Path | Source | What |
| --- | --- | --- |
| `/install` | `src/routes/install/+server.ts` | `pkgs/cli/install.sh` as it is, so the site and the release agree. The file has no extension, which is why the CLI's page is `/cli/`. |
| `/gallery/<code>.tar.gz` | `src/routes/gallery/[slug].tar.gz/` | An example's files from `pkgs/gallery/<code>/`. |
| `/gallery/index.json` | `src/routes/gallery/index.json/` | The published examples, for `aicentre gallery`. |
| `/<path>.md`, `/llms.txt`, `/llms-full.txt` | see above | Markdown for scripts and models. |
| `/sitemap.xml` | `src/routes/sitemap.xml/` | |

## Moved pages

`/clusters/` became `/providers/`. GitHub Pages cannot answer with a
redirect, so `src/lib/server/moved.ts` prerenders each old path as a small
HTML page that forwards to the new one, keeping the query and the
`#fragment`. SvelteKit's own redirect stub drops the fragment, and old links
often point at a heading. Old twin paths (`/clusters/dtu.md`) serve the same
markdown as the new ones.

GitHub Pages serves `404.html` for any path it cannot resolve, and
`adapter-static` writes it as the fallback.

## Build with Nix

```sh
nix build .#website   # into ./result
nix run .#website     # serve it on http://127.0.0.1:8080, or PORT=...
nix flake check       # svelte-check and the site build, the gates CI runs
```

The site's `node_modules` is a fixed-output derivation. When `package.json` or
`bun.lock` changes, `nix build` fails with a hash mismatch. Copy the printed
`got:` hash into `nodeModulesHash` in `flake.nix`.

The hash is recorded per system, because bun resolves platform-specific native
packages (esbuild, rollup, lightningcss, tailwind oxide). A hash updated on a
Mac fixes local builds but not CI, which runs `x86_64-linux`. Take that one
from the failed CI log.

The build patches shebangs in the whole `node_modules` tree. Package entry
points use `#!/usr/bin/env node`, and `/usr/bin/env` does not exist in the
Linux build sandbox. Patching only `.bin` is not enough, because its entries
are symlinks that `patchShebangs` skips.

Nix builds from the git tree. A new file that is not `git add`ed is invisible
to it, and the symptom is a `vite:load-fallback` ENOENT for a file that exists
on disk. `git add -N` is enough.

The website's fileset includes `pkgs/cli/install.sh` and `pkgs/gallery`,
because `/install` and the gallery tarballs read them.

## Deploy

`.github/workflows/deploy.yaml` runs `nix flake check` and
`nix build .#website`, then publishes the result to GitHub Pages. `cli.yaml` tests `pkgs/cli` on every change to it, on Linux and macOS.
`release-cli.yaml` is in [07 · Install and release](07-install-and-release.md).
