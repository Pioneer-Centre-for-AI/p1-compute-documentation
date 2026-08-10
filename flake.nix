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

        # bun resolves platform-specific native packages (esbuild, rollup,
        # lightningcss, tailwind oxide), so the node_modules tree — and its
        # hash — differs per system. One shared hash cannot describe both a mac
        # and the Linux CI runner: whichever system it was not recorded on fails
        # with a mismatch. Record one hash per system instead.
        nodeModulesHash = {
          aarch64-darwin = "sha256-KiT44+6HdBayLOezW+lLseXC78rW3ljdyunvqeO8nTs=";
          x86_64-linux = "sha256-VIiTBdSi7nXPskoJtlOF5xd8MUVt2NLwrF2UeDFayyg=";
        }.${system} or (throw
          "No node_modules hash recorded for ${system}. Run `nix build .#website`, then paste the printed `got:` hash into nodeModulesHash in flake.nix.");

        # Fixed-output derivation: fetches all bun deps over the network and
        # produces a hashed `node_modules` tree. Re-run only when package.json
        # or bun.lock changes — at which point `nix build` will fail with a
        # hash mismatch and print the new hash to paste in above.
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

        # svelte-check as a flake check, so CI gates on type errors without a
        # second dependency install: it reuses the same node_modules FOD as the
        # website build, and produces no output beyond a success marker.
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
            bun run check
            runHook postBuild
          '';

          installPhase = ''
            runHook preInstall
            touch $out
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

        checks.typecheck = typecheck;
        checks.website = website;

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
