# The P1 compute docs

This repository has two kinds of documentation. The site's pages, under
`src/routes/`, are the user manual: what a researcher reads at
<https://hpc.aicentre.dk>. This directory is the technical reference for the
people who maintain the site, the `aicentre` command line tool and the
gallery. Code carries no prose: no explanatory headers, no rationale, no
incident narration. A file's WHY is in the chapter that owns its topic. What
stays in source is what a tool reads: shebangs, `# shellcheck disable` pragmas,
TypeScript directives, Nix option descriptions, and a label of a few words where
a line would otherwise be cryptic.

A page says what a command does for the researcher. It does not explain the
tools behind it, because those change with the implementation. That detail
belongs here.

## Chapters

1. [The site](01-the-site.md): SvelteKit and mdsvex, page collections, markdown
   twins, the prerendered endpoints, building and deploying.
2. [The CLI](02-the-cli.md): what `aicentre` is, its status, the source layout,
   developing it, settings and conventions shared by every command.
3. [Providers and login](03-providers-and-login.md): the clusters, finding the
   Host that reaches each one, the shared ssh login, running on a cluster.
4. [Sessions](04-sessions.md): `shell`, `marimo` and `code`, on a login node or
   with `--gpu` in a job, and the launcher protocol.
5. [Batch and jobs](05-batch-and-jobs.md): `submit`, `logs`, `cancel` and
   `list jobs`.
6. [Check](06-check.md): `aicentre check`, its probes and how it reports.
7. [Install and release](07-install-and-release.md): `install.sh`, the XDG
   layout, `upgrade`, `uninstall`, and cutting a release.
8. [Sync and transfer](08-sync-and-transfer.md): keeping `aicentre` on each
   cluster, and `push` and `pull`.
9. [Chat](09-chat.md): `aicentre chat` and its P1 tools.
10. [Nix](10-nix.md): the package, the Home Manager and NixOS modules, and the
    hashes.
11. [The gallery](11-the-gallery.md): example codes, adding an example, the
    tarballs and `aicentre gallery`.
12. [Testing](12-testing.md): the sandbox, the fakes and the end-to-end run.
