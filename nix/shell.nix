{pkgs ? import <nixpkgs> {config.allowUnfree = true;}, ...}:
pkgs.mkShell rec {
  name = "P1 Compute Documentation";
  buildInputs = [
    pkgs.uv
    # pkgs.pixi
    pkgs.wget
    pkgs.mypy
    pkgs.shfmt
    pkgs.shellcheck
    pkgs.just
    pkgs.mdformat
  ];
}
