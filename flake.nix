{
  description = "P1 HPC documentation site (SvelteKit + bun)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    # aicentre (pkgs/cli) needs bun 1.4, whose bun.lock format older bun cannot
    # read. Its own pin keeps the website's nixpkgs, and its hashes, as they are.
    nixpkgs-cli.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, nixpkgs-cli, flake-utils }:
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
              # Served at /install by src/routes/install/+server.ts.
              ./pkgs/cli/install.sh
              # Served as /gallery/<code>.tar.gz by src/routes/gallery/[slug].tar.gz.
              ./pkgs/gallery
            ];
          };

          nativeBuildInputs = [ pkgs.bun pkgs.nodejs_22 ];

          buildPhase = ''
            runHook preBuild
            export HOME=$(mktemp -d)
            cp -R ${nodeModules}/node_modules ./node_modules
            chmod -R u+w node_modules
            # Package entry points carry `#!/usr/bin/env node` shebangs, and
            # /usr/bin/env does not exist inside the Linux build sandbox. The
            # whole tree has to be patched rather than just .bin, whose entries
            # are symlinks that patchShebangs skips.
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
            # Package entry points carry `#!/usr/bin/env node` shebangs, and
            # /usr/bin/env does not exist inside the Linux build sandbox. The
            # whole tree has to be patched rather than just .bin, whose entries
            # are symlinks that patchShebangs skips.
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

        # The aicentre command line tool, from pkgs/cli.
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
            # For pkgs/cli/install.sh: `bun run lint:sh` in pkgs/cli.
            shellcheck
          ];

          shellHook = ''
            echo "P1 HPC dev shell — bun $(bun --version), node $(node --version)"
          '';
        };

        formatter = pkgs.nixpkgs-fmt;
      })
    // {
      # aicentre for Nix users: a Home Manager module with settings, a NixOS
      # module that installs it for everyone, and an overlay with pkgs.aicentre.
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
