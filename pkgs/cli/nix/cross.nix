# Development only: the cluster's build of a checkout. Releases come from CI.
{ rustBin
, cargo-zigbuild
, zig
, git
, writeShellApplication
, mkShell
}:

let
  target = "x86_64-unknown-linux-musl";
  rust = rustBin.stable.latest.minimal.override { targets = [ target ]; };
in
rec {
  tools = [ rust cargo-zigbuild zig git ];

  build = writeShellApplication {
    name = "aicentre-static-x64";
    runtimeInputs = tools;
    text = ''
      root=$(git rev-parse --show-toplevel)
      cd "$root/pkgs/cli"
      cargo zigbuild --release --locked --target ${target} "$@" >&2
      echo "$root/pkgs/cli/target/${target}/release/aicentre"
    '';
  };

  shell = mkShell {
    packages = tools;
    shellHook = ''
      echo "Cross shell: cargo zigbuild --release --target ${target}"
    '';
  };
}
