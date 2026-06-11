{
  description = "P1 HPC documentation site (SvelteKit + bun)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        # Fixed-output derivation: fetches all bun deps over the network and
        # produces a hashed `node_modules` tree. Re-run only when package.json
        # or bun.lock changes — at which point `nix build` will fail with a
        # hash mismatch and print the new hash to paste in below.
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
          outputHash = "sha256-uV6sjPRhRda5DXvUfsgD0Bo52xUN9k8XaKMZI8o5O+U=";
        };

        website = pkgs.stdenvNoCC.mkDerivation {
          pname = "p1-hpc-website";
          version = "0.1.0";

          src = pkgs.lib.fileset.toSource {
            root = ./.;
            fileset = pkgs.lib.fileset.unions [
              ./package.json
              ./bun.lock
              ./svelte.config.js
              ./vite.config.ts
              ./tsconfig.json
              ./src
              ./static
            ];
          };

          nativeBuildInputs = [ pkgs.bun pkgs.nodejs_22 ];

          buildPhase = ''
            runHook preBuild
            export HOME=$(mktemp -d)
            cp -R ${nodeModules}/node_modules ./node_modules
            chmod -R u+w node_modules
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

        apps.website = {
          type = "app";
          program = "${serveScript}";
        };
        apps.default = self.apps.${system}.website;

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            bun
            nodejs_22
          ];

          shellHook = ''
            echo "P1 HPC dev shell — bun $(bun --version), node $(node --version)"
          '';
        };

        formatter = pkgs.nixpkgs-fmt;
      });
}
