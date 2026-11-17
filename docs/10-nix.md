# Nix

The root `flake.nix` builds the site ([01](01-the-site.md#build-with-nix)) and
aicentre, and exports aicentre's modules.

```sh
nix build .#aicentre
nix run .#aicentre -- check
```

## The package

`pkgs/cli/nix/package.nix` builds aicentre from source with
`rustPlatform.buildRustPackage`. The dependencies come from `Cargo.lock`
through `cargoLock.lockFile`, so a change to them needs no hash. The version is
the one in `Cargo.toml`.

The check phase runs `cargo test`: the unit tests and the scenario tests
against the fake ssh, with `sh`, `bash` and `tar` from the build environment,
and `python3`, `procps` and `openssh` for the launcher tests. Two launcher
tests are skipped there, because the sandbox cannot resolve a host name
([12](12-testing.md#ci-and-nix)). The end-to-end tests need `AICENTRE_E2E=1`
and stay off.

The release binary links musl statically. This build links against the Nix
glibc, like any other package. `aicentre upgrade` and `uninstall` refuse to
touch a binary in the Nix store, because Nix owns it. The output is
`bin/aicentre`.

The package builds on x86_64-linux, aarch64-linux and aarch64-darwin.
nixpkgs-unstable no longer supports x86_64-darwin, so Intel Macs use the
release binary. The `nix` job in `.github/workflows/cli.yaml` builds the
package on all three systems whenever `pkgs/cli`, `flake.nix` or
`flake.lock` changes.

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
reads. Everything else aicentre writes stays in its own files.

`sshInclude` (default `true`) adds `${xdg.stateHome}/aicentre/ssh_config` to
`programs.ssh.includes` when `programs.ssh.enable` is set. That is the one
line [desktop VS Code on a GPU node](04-sessions.md#on-a-gpu-node) needs, and
Home Manager puts it at the top of `~/.ssh/config`, before any Host. aicentre
writes the file itself; the module does not.

`nixosModules.default` (also `.aicentre`), from `nix/nixos.nix`:
`programs.aicentre.enable` installs aicentre for every user. Per-user
settings belong in the Home Manager module, or in each user's
`config.toml`.

`overlays.default` adds `pkgs.aicentre`, built with the `rustPlatform` of the
nixpkgs it overlays.
