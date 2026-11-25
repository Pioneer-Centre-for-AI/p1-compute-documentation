# P1 compute documentation

The site at [hpc.aicentre.dk](https://hpc.aicentre.dk) for P1 compute, and what it ships:

- `src/`, `static/`: the site, SvelteKit on GitHub Pages.
- `pkgs/cli/`: `aicentre`, the command line tool.
- `pkgs/gallery/`: the files of the gallery's examples.

## Develop

```sh
nix develop
bun install
bun run dev                      # the site, http://localhost:5173/

cd pkgs/cli
cargo test
nix run .#static-x64             # the cluster's build, which sessions of cargo run use
cargo run -- marimo
```

## Release

Set `version` in `pkgs/cli/Cargo.toml`, merge, then push a tag: `cli-v0.2.0`, or
`cli-v0.2.0-rc.1` for a pre-release.
