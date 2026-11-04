# P1 compute documentation

The site at [hpc.aicentre.dk](https://hpc.aicentre.dk) for P1 compute, and what it ships:

- `src/`, `static/`: the site, SvelteKit on GitHub Pages. Its pages are the user manual.
- `pkgs/cli/`: `aicentre`, the command line tool installed from `https://hpc.aicentre.dk/install`.
- `pkgs/gallery/`: the files of the gallery's examples, served as `/gallery/<code>.tar.gz`.
- `docs/`: the technical reference for maintainers. Start at [docs/README.md](docs/README.md).

## Develop

```sh
nix develop    # bun, node and shellcheck; or `direnv allow` once
bun install
bun run dev    # http://localhost:5173/
bun run check
bun run build
```

The CLI has its own commands in `pkgs/cli`; see [docs/02-the-cli.md](docs/02-the-cli.md).
