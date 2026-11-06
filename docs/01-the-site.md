# The site

The site at <https://hpc.aicentre.dk> is SvelteKit 3 with `adapter-static`,
built to `build/` and served by GitHub Pages. Pages are mdsvex `.svx` files
under `src/routes/`. bun is the package manager.

SvelteKit 3 has no `svelte.config.js`. The site config (adapter, mdsvex,
extensions) is passed to the `sveltekit()` plugin in `vite.config.ts`.
`version.pollInterval` is 0, because a static site has no new version to
poll for between deploys. Code in `src/` imports `src/lib` through the
`#lib/*` subpath import in `package.json`, with the file extension:
`#lib/links.ts`, `#lib/components/Meta.svelte`. Param matchers live in
`src/params.ts`.

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
- Event pages, `src/routes/events/<slug>/+page.svx`, through
  `src/lib/events.ts`.

`ClusterQuotas` takes a slug, not values, so a page cannot state a limit that
disagrees with its frontmatter. `QUOTA_GROUPS` in `content.ts` holds the order
and the labels, and both the component and the markdown twin read it.

Tables that a page and its twin both show live in the `src/lib` modules, not
in the page: the provider decision table and the compute tiers in
`content.ts`, and the D3A programme, organizers and collaborations in
`events.ts`. A `.svx` script block is plain JS and cannot carry type
assertions, so joins against the provider registry happen in `content.ts`
too. A compute tier's `description` is inline HTML written in the repository
and rendered with `{@html}`. Do not feed it anything from outside.

### Import cycles

Each collection globs its pages, and the pages import components that import
the collection back. Vite's dev SSR runner walks that cycle live, so a page's
`metadata` export can still be undefined while its module evaluates. The
rules that keep this working:

- Parse on first call, never at module scope: `getClusters()`, `getEvents()`
  and `getGallery()`. An eager parse threw a ZodError that took the nav down.
- The cache is a `var`. The dev runner can hand a re-entering module the
  partial namespace, so a caller can reach the getter before the declaration
  has run. A `var` is hoisted as `undefined` and `??=` fills it. A `let`
  throws "Cannot access before initialization".
- `getNav()` in `src/lib/nav.ts` is a function for the same reason. Call
  these while rendering, not at module scope.

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
`## Optional`. Every link in it points at a twin, not at the HTML. Titles and
descriptions come from the page's `<Meta>` call in the source, because a page
source has no `<title>` or `<meta>` tag to read. The landing page is a
`+page.svelte` with no twin. Its description lives in `src/lib/site.ts`, and
`llms.txt` and the sitemap add it by hand. The sitemap reuses
`MARKDOWN_PAGES`, so a page cannot reach it without passing the drift check.

`llms-full.txt` puts each page's canonical URL above its twin and makes every
link absolute, because the file gets pasted where there is no base URL.

`src/lib/markdown/toMarkdown.ts` turns a `.svx` source into markdown in
steps:

1. Drop the frontmatter.
2. Set fenced code aside, so `<your-username>` in a shell example is not
   stripped as a tag.
3. Swap each `<!-- md:replace NAME -->` ... `<!-- md:end -->` region for
   markdown built by `twins.ts` from the same registry the page loops over.
   A source transform cannot follow a Svelte loop.
4. Drop scaffolding. `Aside` becomes a blockquote, `EmbedForm` and its
   wrappers become a link line, `Support` becomes a contact list.
   `ClusterQuotas` gets its values from the registry, like the hardware
   table.
5. Drop any component that is left, rather than emit raw Svelte.
6. Resolve `p1:` links the way the remark plugin does, then restore the code.

Provider and event pages get their H1 from the layout, and provider pages
also get their hardware table there, so `twins.ts` adds both back.

The `[...path].md` route compiles to `([^]*?)\.md`, so one route covers
nested pages. Endpoints default to `trailingSlash: 'never'`, so `/about.md`
resolves without a redirect. adapter-static runs with `strict: true`. Its
crawler cannot find an endpoint nothing links to, so the glob in `twins.ts`
feeds both the prerender entries and the bodies.

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

- The forwarding response is exactly `text/html`. SvelteKit only saves a
  prerendered endpoint as `<path>/index.html` for that type.
- `/clusters/` sets `trailingSlash: 'always'`, because the old URL ended in a
  slash and endpoints default to `'never'`.
- `/clusters/[slug=segment]/` makes one page per provider, so a provider added
  later gets an old-style URL too. The `segment` matcher in `src/params.ts`
  takes one path segment with no dot, which leaves `/clusters/dtu.md` to the
  markdown route.

## Links

Every external URL is in `src/lib/urls.js`. It is plain JS because Vite loads
`vite.config.ts`, which imports it through the mdsvex plugins. `src/lib/links.ts` re-exports the URLs
as named constants for components.

In a `.svx` page, write `[workload survey](p1:survey)`. A raw
`<a href={CONST}>` at the start of a line ends the paragraph, so the rest of
the sentence falls out of the `<p>`. The remark plugin in
`src/lib/markdown/urlScheme.js` resolves `p1:<key>`, throws on an unknown
key, and runs before `rehype-external-links`, so external links still get
`target` and `rel`.

The forms are on Microsoft Forms under the institution, so they outlive
whoever holds the coordinator role. Two are not. The access exception form is
on Google Forms. Its short link is the one to share, and the embed takes the
full URL with Google's embed parameter. The booking calendar belongs to a
person, so the next coordinator replaces its URL. Calls to action point at
`/survey/`, which embeds the survey in the site, not at the raw form. The
`/forms/` route keeps its path although the page is called Forms & Bookings,
so bookmarks and the sitemap keep working.

## Code blocks

`src/lib/markdown/highlight.js` renders fenced blocks with shiki at build
time, through the mdsvex highlighter in `vite.config.ts`. A metastring adds
a file name to the header: ` ```bash file=train.sh `. The header bar is always
drawn, so the copy button has its own row and does not cover the first line
on a phone.

The block is raw HTML inserted with `{@html}`. Its styles therefore live in
`src/app.css`, not in a scoped component style, and the copy buttons have no
handler of their own. One listener in the root layout handles every block,
including blocks that appear after client-side navigation. It sets
`data-copied`, which swaps the icon. Dark mode flips only shiki's `color`
variables. Setting `background-color` on the token spans as well drew a box
around every token.

## Styling

`src/app.css` holds the theme. Its points are easy to undo by mistake:

- The slate ramp is replaced with P1's ink tones, so every `slate-*` utility
  takes the brand tone. Do not put stock slate back.
- Coral is decoration only (shapes, accent bars). It never fills a button or
  colours body text. Plum carries interactive colour, and `highlight` is the
  yellow marker from aicentre.dk headlines.
- `--color-brand` flips with the theme, so links stay at WCAG AA contrast.
  `color-scheme` follows the site's own toggle, so scrollbars and native
  controls match it.
- Tailwind v4's preflight dropped the pointer cursor on buttons. The site puts
  it back, because links and buttons are styled alike.
- The prose rules carry `:not(.not-prose, .not-prose *)`.
  `@tailwindcss/typography` is not installed, so `not-prose` means only what
  these guards make it mean. The rules are unlayered and beat Tailwind
  utilities, so a rule without the guard overrides component styles.
- The diagrams lift `slate-500` text in dark mode. It is about 3.4:1 on
  `slate-900`, too low for 10 to 12 px text.
- Movement runs only for people who have not asked for less motion: hover
  lifts and nudges use `motion-safe:`, and the compute-needs menu slides by
  `prefersReducedMotion` from `svelte/motion`. Colour and shadow changes stay.
- The layout renders the one `<main id="main">` for every page, and a "Skip to
  content" link before the header targets it. Pages do not add their own
  `<main>`.
- The event notice bar adds about 28 px to the header. The sidebar's
  `top-24` and the headings' `scroll-mt-24` are sized for it.

## Components

Notes on components whose shape is not obvious from the code:

- `BookingDialog` and `EmbedForm` use a native `<dialog>` with `showModal()`.
  The sticky header has `backdrop-filter`, which makes it the containing block
  for `fixed` children. A `fixed inset-0` overlay opened from the header hung
  off the top of the page. The top layer is out of reach of any ancestor. The
  iframe's `src` stays empty until the first open, so Cal and the forms do
  not load on every page view. The `close` event is the one place state is
  reset. The booking dialog is 72rem wide, because Cal shows its three
  columns only in a wide iframe, and it reads the theme when it opens.
- `ComputeNeedsMenu` is a split button. The label goes to the survey and the
  caret opens booking. It closes on `pointerdown` in the capture phase and
  has the arrow keys that `role="menu"` promises.
- `Meta` sets every per-page head tag, including Open Graph and Twitter, so
  a link pasted into Slack shows a card. A page that passes `image` passes
  `imageAlt` too.
- `EventNotice` decides at build time, so the prerendered HTML and the first
  render agree. A build that outlives the event hides the bar after
  hydration.
- The events layout writes the schema.org `Event` JSON-LD in markup, because
  a literal closing script tag inside `<script>` would end the block.
- `ScaledEmbed` renders an iframe at its natural width and scales it to the
  column, for the D3A Airtable board. Below `minScale` it scrolls instead.
- `ResearchNetworkDiagram` shows only facilities this site documents. Edit
  its two lists to add or remove one.
- `GpuAllowance` frames the form around protecting a run. Where a cluster
  lets people pass the default limit themselves (Gefion), that is faster.
  `selfServeDocumented` hides the generic pointer on such pages.
- Full-width pages, such as the gallery overview, skip the sidebar shell. The
  providers, events and gallery layouts render their overview pages as they
  are. `PageToc` hides on pages with fewer than two headings.
- Terms and Privacy are only in the footer.

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
`nix build .#website`, then publishes the result to GitHub Pages. The flake
check runs svelte-check and the site build, so pull requests are gated on type
errors, the Zod frontmatter schemas and the `MARKDOWN_PAGES` drift guard.

Before upload, the workflow fails if the output lacks any entry point:
`index.html`, `404.html`, `sitemap.xml`, `robots.txt`, `CNAME`, `install`,
`gallery/index.json` or `gallery/int-gpu.tar.gz`. It also fails if `install`
does not start with `#!/bin/sh`, since `curl | sh` runs that file.

The workflow reads the repository and nothing else. Only the deploy job gets
the Pages permissions, so a pull request from a fork cannot obtain them. A
new push to a pull request cancels its older run. Pushes to `main` never
cancel a deploy in progress.

The cachix step reads the public `nix-community` cache, which covers bun and
node but not this project's derivations. To cache the `node_modules`
derivation and the site build across runs, create a cache of your own, set
`CACHIX_AUTH_TOKEN` in the repository secrets and point the step at it.

`cli.yaml` tests `pkgs/cli` on Linux and macOS and builds the Nix package
([10 · Nix](10-nix.md)). `release-cli.yaml` is in
[07 · Install and release](07-install-and-release.md).
