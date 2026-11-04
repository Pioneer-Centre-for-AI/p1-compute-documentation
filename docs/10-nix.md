# Nix

The root `flake.nix` builds the site ([01](01-the-site.md#build-with-nix)) and
aicentre, and exports aicentre's modules.

```sh
nix build .#aicentre
nix run .#aicentre -- check
```

## The package

`pkgs/cli/nix/package.nix` builds aicentre from source with the bun from the
flake's `nixpkgs-cli` input. aicentre's `bun.lock` needs bun 1.4, whose lock
format older bun cannot read. A separate input keeps the website's nixpkgs,
and its hashes, as they are.

Unlike the release binary, this build links against the Nix glibc, so it runs
on NixOS without nix-ld. `aicentre upgrade` and `uninstall` refuse to touch a
binary in the Nix store, because Nix owns it. The output is
`libexec/aicentre/` (the binary and `pi/`) with a link in `bin/`.

The dependencies are a fixed-output derivation, the only step with network
access: `bun install --frozen-lockfile --production`. aicentre's production
dependencies have no platform-specific packages, so one `nodeModulesHash`
covers every system. After a change to `bun.lock` the build fails with the
new hash to paste in. The derivation has
`dontFixup = true`. Patching shebangs would make a fixed output refer to the
store, which Nix rejects.

The final derivation sets `dontStrip`, `dontPatchELF` and `dontFixup`. The
program is appended to the bun runtime inside the executable, and strip or
patchelf would cut it off.

The package builds on x86_64-linux, aarch64-linux and aarch64-darwin.
nixpkgs-unstable no longer supports x86_64-darwin, so Intel Macs use the
release binary. The `nix` job in `.github/workflows/cli.yaml` builds the
package on all three systems whenever `pkgs/cli`, `flake.nix` or
`flake.lock` changes, so a stale hash fails there first.

Nix sees only files git knows about, so `git add` (or `git add -N`) new files
first.

## Modules

`homeManagerModules.default` (also `.aicentre`), from `nix/home-manager.nix`:

```nix
programs.aicentre = {
  enable = true;
  settings = {
    dtu.user = "s123456";
    gefion.user = "jdoe";
  };
};
```

`settings` becomes `~/.config/aicentre/config.toml`, which aicentre only
reads. Gefion jobs are charged to P1's `iu_0096` unless `gefion.account` says
otherwise. Chat's model logins and everything else aicentre writes stay in
their own files.

`nixosModules.default` (also `.aicentre`), from `nix/nixos.nix`:
`programs.aicentre.enable` installs aicentre for every user. Per-user
settings belong in the Home Manager module, or in each user's
`config.toml`.

`overlays.default` adds `pkgs.aicentre`, built with `nixpkgs-cli`'s bun.
