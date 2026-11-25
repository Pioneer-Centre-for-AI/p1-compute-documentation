{
  description = "P1 HPC: the documentation site and the aicentre CLI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    # Development only: cross builds for the clusters.
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ];

      forEachSystem = outputs: nixpkgs.lib.genAttrs systems (system:
        let pkgs = nixpkgs.legacyPackages.${system}; in
        outputs {
          inherit pkgs;
          site = pkgs.callPackage ./nix/website.nix { };
          aicentre = pkgs.callPackage ./pkgs/cli/nix/package.nix { };
          cross = pkgs.callPackage ./pkgs/cli/nix/cross.nix {
            rustBin = rust-overlay.lib.mkRustBin { } pkgs;
          };
        });

      app = program: { type = "app"; inherit program; };
    in
    {
      packages = forEachSystem ({ site, aicentre, ... }: {
        inherit (site) website;
        inherit aicentre;
        default = site.website;
      });

      checks = forEachSystem ({ site, ... }: {
        inherit (site) website typecheck;
      });

      apps = forEachSystem ({ site, aicentre, cross, ... }: {
        website = app "${site.serve}";
        default = app "${site.serve}";
        aicentre = app "${aicentre}/bin/aicentre";
        static-x64 = app "${cross.build}/bin/aicentre-static-x64";
      });

      devShells = forEachSystem ({ pkgs, cross, ... }: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            bun
            nodejs_22
            cargo
            rustc
            clippy
            rustfmt
            rust-analyzer
            shellcheck
            secretspec
          ];

          shellHook = ''
            echo "P1 HPC dev shell: bun $(bun --version), node $(node --version), $(rustc --version)"
          '';
        };

        cross = cross.shell;
      });

      formatter = forEachSystem ({ pkgs, ... }: pkgs.nixpkgs-fmt);

      overlays.default = final: _prev: {
        aicentre = final.callPackage ./pkgs/cli/nix/package.nix { };
      };

      nixosModules = rec {
        default = import ./pkgs/cli/nix/nixos.nix self;
        aicentre = default;
      };

      homeManagerModules = rec {
        default = import ./pkgs/cli/nix/home-manager.nix self;
        aicentre = default;
      };
    };
}
