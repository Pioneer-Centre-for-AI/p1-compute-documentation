{
  description = "P1 HPC documentation site (SvelteKit + bun)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs-cli.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, nixpkgs-cli, flake-utils }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ] (system:
      let
        pkgs = import nixpkgs { inherit system; };

        nodeModulesHash = {
          aarch64-linux = "sha256-D6qiX9h+s45aHQEw3SEFKIVPflhknwAl54F4eoDqTz8=";
          aarch64-darwin = "sha256-sQZi8cujOoeNM9bZBR+n2cl1aNHTWEo6UxdciJFzn4w=";
          x86_64-linux = "sha256-gJL+Gpsn4py9rxemFhESTElCtR+p5HFGbVr00T3rUbw=";
        }.${system} or (throw
          "No node_modules hash recorded for ${system}. Run `nix build .#website`, then paste the printed `got:` hash into nodeModulesHash in flake.nix.");

        nodeModules = pkgs.stdenvNoCC.mkDerivation {
          pname = "p1-hpc-node-modules";
          version = "0.1.0";

          src = pkgs.lib.fileset.toSource {
            root = ./.;
            fileset = pkgs.lib.fileset.unions [
              ./package.json
              ./bun.lock
            ];
          };

          nativeBuildInputs = [ pkgs.bun ];

          dontConfigure = true;

          buildPhase = ''
            runHook preBuild
            export HOME=$(mktemp -d)
            bun install --frozen-lockfile --no-progress
            runHook postBuild
          '';

          installPhase = ''
            runHook preInstall
            mkdir -p $out
            cp -R node_modules $out/
            runHook postInstall
          '';

          outputHashMode = "recursive";
          outputHashAlgo = "sha256";
          outputHash = nodeModulesHash;
        };

        website = pkgs.stdenvNoCC.mkDerivation {
          pname = "p1-hpc-website";
          version = "0.1.0";

          src = pkgs.lib.fileset.toSource {
            root = ./.;
            fileset = pkgs.lib.fileset.unions [
              ./package.json
              ./bun.lock
              ./vite.config.ts
              ./tsconfig.json
              ./src
              ./static
              ./pkgs/cli/install.sh
              ./pkgs/gallery
            ];
          };

          nativeBuildInputs = [ pkgs.bun pkgs.nodejs_22 ];

          buildPhase = ''
            runHook preBuild
            export HOME=$(mktemp -d)
            cp -R ${nodeModules}/node_modules ./node_modules
            chmod -R u+w node_modules
            patchShebangs node_modules
            bun run build
            runHook postBuild
          '';

          installPhase = ''
            runHook preInstall
            mkdir -p $out
            cp -R build/. $out/
            runHook postInstall
          '';
        };

        typecheck = pkgs.stdenvNoCC.mkDerivation {
          pname = "p1-hpc-typecheck";
          version = "0.1.0";

          src = website.src;

          nativeBuildInputs = [ pkgs.bun pkgs.nodejs_22 ];

          buildPhase = ''
            runHook preBuild
            export HOME=$(mktemp -d)
            cp -R ${nodeModules}/node_modules ./node_modules
            chmod -R u+w node_modules
            patchShebangs node_modules
            bun run check
            runHook postBuild
          '';

          installPhase = ''
            runHook preInstall
            touch $out
            runHook postInstall
          '';
        };

        aicentre = nixpkgs-cli.legacyPackages.${system}.callPackage ./pkgs/cli/nix/package.nix { };

        serveScript = pkgs.writeShellScript "serve-p1-hpc" ''
          exec ${pkgs.miniserve}/bin/miniserve \
            --index index.html \
            --port "''${PORT:-8080}" \
            --interfaces 127.0.0.1 \
            ${website}
        '';
      in
      {
        packages.website = website;
        packages.default = website;
        packages.aicentre = aicentre;

        checks.typecheck = typecheck;
        checks.website = website;

        apps.website = {
          type = "app";
          program = "${serveScript}";
        };
        apps.default = self.apps.${system}.website;
        apps.aicentre = {
          type = "app";
          program = "${aicentre}/bin/aicentre";
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            bun
            nodejs_22
            shellcheck
          ];

          shellHook = ''
            echo "P1 HPC dev shell — bun $(bun --version), node $(node --version)"
          '';
        };

        formatter = pkgs.nixpkgs-fmt;
      })
    // {
      homeManagerModules.default = import ./pkgs/cli/nix/home-manager.nix self;
      homeManagerModules.aicentre = self.homeManagerModules.default;
      nixosModules.default = import ./pkgs/cli/nix/nixos.nix self;
      nixosModules.aicentre = self.nixosModules.default;
      overlays.default = final: _prev: {
        aicentre = final.callPackage ./pkgs/cli/nix/package.nix {
          inherit (nixpkgs-cli.legacyPackages.${final.stdenv.hostPlatform.system}) bun;
        };
      };
    };
}
