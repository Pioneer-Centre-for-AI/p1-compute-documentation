{ lib
, stdenvNoCC
, bun
, nodejs_22
, miniserve
, writeShellScript
}:

let
  version = "0.1.0";
  system = stdenvNoCC.hostPlatform.system;

  # bun's node_modules differ by platform, so each has its own hash.
  nodeModulesHash = {
    aarch64-linux = "sha256-D6qiX9h+s45aHQEw3SEFKIVPflhknwAl54F4eoDqTz8=";
    aarch64-darwin = "sha256-sQZi8cujOoeNM9bZBR+n2cl1aNHTWEo6UxdciJFzn4w=";
    x86_64-linux = "sha256-gJL+Gpsn4py9rxemFhESTElCtR+p5HFGbVr00T3rUbw=";
  }.${system} or (throw
    "No node_modules hash recorded for ${system}. Run `nix build .#website`, then paste the printed `got:` hash into nodeModulesHash in nix/website.nix.");

  nodeModules = stdenvNoCC.mkDerivation {
    pname = "p1-hpc-node-modules";
    inherit version;

    src = lib.fileset.toSource {
      root = ./..;
      fileset = lib.fileset.unions [
        ../package.json
        ../bun.lock
      ];
    };

    nativeBuildInputs = [ bun ];

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

  src = lib.fileset.toSource {
    root = ./..;
    fileset = lib.fileset.unions [
      ../package.json
      ../bun.lock
      ../vite.config.ts
      ../tsconfig.json
      ../src
      ../static
      ../pkgs/cli/install.sh
      ../pkgs/gallery
    ];
  };

  bunRun = { pname, script, installPhase }: stdenvNoCC.mkDerivation {
    inherit pname version src installPhase;

    nativeBuildInputs = [ bun nodejs_22 ];

    buildPhase = ''
      runHook preBuild
      export HOME=$(mktemp -d)
      cp -R ${nodeModules}/node_modules ./node_modules
      chmod -R u+w node_modules
      patchShebangs node_modules
      bun run ${script}
      runHook postBuild
    '';
  };
in
rec {
  website = bunRun {
    pname = "p1-hpc-website";
    script = "build";
    installPhase = ''
      runHook preInstall
      mkdir -p $out
      cp -R build/. $out/
      runHook postInstall
    '';
  };

  typecheck = bunRun {
    pname = "p1-hpc-typecheck";
    script = "check";
    installPhase = ''
      runHook preInstall
      touch $out
      runHook postInstall
    '';
  };

  serve = writeShellScript "serve-p1-hpc" ''
    exec ${miniserve}/bin/miniserve \
      --index index.html \
      --port "''${PORT:-8080}" \
      --interfaces 127.0.0.1 \
      ${website}
  '';
}
