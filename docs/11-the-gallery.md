# The gallery

The gallery at <https://hpc.aicentre.dk/gallery/> is a set of example
workflows for P1-affiliated researchers. Each example has a page under
`src/routes/gallery/<code>/` and, when it brings files, a directory under
`pkgs/gallery/<code>/`. The page is the example's manual. It describes the
example and how `aicentre` runs it.

| Code | What it shows | Providers | Files |
| --- | --- | --- | --- |
| `int-gpu` | An interactive session with a GPU: a shell, marimo or VS Code | DTU (`p1i`) | a marimo notebook, a GPU script, `pyproject.toml` and `uv.lock` |
| `rem-lgn` | VS Code on a login node over Remote-SSH | DTU, Gefion | `ssh_config`, `settings.json`, `tasks.json` |
| `xfr-dat` | Moving code and data to and from the clusters | DTU, Gefion, NGC | none, commands only |

## Codes

Every example has a code of the form `TYP-SCO`: a three-letter session type and
a three-letter scope. It is the directory name and the page's URL in lowercase
(`int-gpu`, `/gallery/int-gpu/`), and the label on its card in uppercase
(`INT-GPU`). The types are a fixed list, `SESSION_TYPES` in
`src/lib/gallery.ts`:

| Type | Meaning |
| --- | --- |
| `INT` | Interactive: someone at the keyboard on a compute node |
| `REM` | Remote: an editor connected to a login node |
| `BAT` | Batch: a job submitted to a batch queue |
| `XFR` | Transfer: moving code and data |
| `SRV` | Service: a server called from a laptop |

The scope names the resource or subject, such as `GPU`, `LGN` for a login
node, `DAT` for data, or a provider.

## Adding an example

1. Pick a code. When the example brings files (code, a notebook, locked
   packages, settings), add them under `pkgs/gallery/<code>/`. Add no README:
   the page is the manual. When the example needs something the CLI cannot do
   yet, that is a sign for the CLI.
2. Add its page as `src/routes/gallery/(drafts)/<code>/+page.svx`. Copy the
   frontmatter of an existing page. `src/lib/gallery.ts` validates it, and the
   code, at build time:
   - `title`, `description`, `navLabel`, and `summary` (one sentence for the
     card).
   - `order`: the position in the gallery.
   - `tags`: lowercase words that become the filter chips.
   - `clusters`: any of `dtu`, `gefion`, `ngc`.
   - `thumbnail` (optional): `{ src: /assets/gallery/<code>.avif, alt: ... }`.
     Without one the card draws the code's illustration from
     `src/lib/components/GalleryIllustration.svelte`, or a plain GPU.
   - `commands` (optional): what to run once the files are on the cluster, as
     `aicentre gallery get` prints them. Paths assume its default,
     `examples/<code>` in the project directory.
3. Publish it: move the page up to `src/routes/gallery/<code>/` and add
   `gallery/<code>` to `MARKDOWN_PAGES` in `src/lib/markdownPages.ts`. The
   build fails if the second step is forgotten.

Drafts show in `bun run dev` with a Draft badge. Their own `+layout.ts` turns
prerendering off and answers 404 outside dev, so they do not exist in
production. The globs that collect gallery pages match the `(drafts)` folder
with character classes, `[(]drafts[)]`, because parentheses are glob syntax.

## Tarballs and the index

`src/lib/server/gallery-files.ts` reads every file under
`pkgs/gallery/<code>/` at build time. `/gallery/<code>.tar.gz` is prerendered
for each published example that has files. An example with no files gets no
tarball. `src/lib/server/tar.ts` is a small ustar writer that gives every
entry the same owner, mode and time, so the same files always give the same
bytes, as a reproducible build needs. Entries sit under `<code>/`.

A path of up to 100 bytes goes in the ustar `name` field. A longer one is
split into `prefix` and `name`. Without aicentre, an example unpacks with
`curl -fsSL https://hpc.aicentre.dk/gallery/<code>.tar.gz | tar -xz`.

`/gallery/index.json` lists the published examples: code, slug, title,
summary, providers, page URL, the tarball URL or `null`, and `commands`.

## The gallery page

`GalleryBrowser` holds the search box, the provider and tag filters, and the
card grid. The prerendered page shows every card, so it works before
hydration and without JavaScript. The filters only narrow it. The chosen
provider and tag go in `?provider=` and `?tag=`, so other pages can link to a
filtered gallery. They are read after mount, because a prerendered page has
no query string at build time. Every provider shows, in registry order. One
with no examples is disabled with a count of zero.

A card is a `div`, not one big link, because it holds several links. The
thumbnail link repeats the title link, so it is kept out of the tab order and
the accessibility tree. With `onTag` the tags are filter buttons. The landing
page leaves it out and gets plain labels. `GalleryIllustration` draws in the
same style as `ClusterIllustration`: strokes in `currentColor`, coral as
decoration, and `--color-brand` for the accent. INT-GPU's drawing shows the
three spirals its notebook trains on.

The sidebar does not list the gallery, so the example layout has its own link
back. The header link to the gallery shows on every screen size.

A gallery card's Source button links to `pkgs/gallery/<code>` on GitHub, and
is left out when the example has no directory. The card also renders in the
browser, where the server-only file glob is not available, so
`vite.config.ts` lists the directories at build time as `__GALLERY_DIRS__`.
A new directory shows up in `bun run dev` after a restart.

## aicentre gallery

`aicentre gallery` lists the index. `aicentre gallery get <code> [dir]` puts an
example on a cluster (`src/gallery/gallery.ts`). The login node downloads the
tarball from the site itself, with curl or with wget where there is no curl.
Nothing passes through the laptop, and nobody has to clone the repository. It
unpacks into `examples/<code>` in the project directory, or under `dir`, and
prints the example's `commands`. An example without files prints its
commands only. `AICENTRE_SITE` points the command at another site, including
a `file://` directory, which the tests use.
