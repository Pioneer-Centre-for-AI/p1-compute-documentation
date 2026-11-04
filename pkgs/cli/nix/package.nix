{ lib
, stdenvNoCC
, bun
, nodeModulesHash ? "sha256-HpMXuDC/vvzC5Yl1LNU9wR1pWHKiXXQIAeIfUh6eesA="
,
}:

let
  pkg = lib.importJSON ../package.json;

  src = lib.fileset.toSource {
    root = ./..;
    fileset = lib.fileset.unions [
      ../package.json
      ../bun.lock
      ../tsconfig.json
      ../scripts
      ../src
    ];
  };

  nodeModules = stdenvNoCC.mkDerivation {
    pname = "aicentre-node-modules";
    inherit (pkg) version;
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
      bun install --frozen-lockfile --no-progress --production
      runHook postBuild
    '';
    installPhase = ''
      runHook preInstall
      mkdir -p $out
      cp -R node_modules $out/
      runHook postInstall
    '';
    dontFixup = true;
    outputHashMode = "recursive";
    outputHashAlgo = "sha256";
    outputHash = nodeModulesHash;
  };
in
stdenvNoCC.mkDerivation {
  pname = "aicentre";
  inherit (pkg) version;
  inherit src;

  nativeBuildInputs = [ bun ];
  dontConfigure = true;

  buildPhase = ''
    runHook preBuild
    export HOME=$(mktemp -d)
    cp -R ${nodeModules}/node_modules ./node_modules
    chmod -R u+w node_modules
    bun run scripts/build.ts --outfile build/aicentre
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    mkdir -p $out/libexec/aicentre $out/bin
    cp -R build/. $out/libexec/aicentre/
    ln -s $out/libexec/aicentre/aicentre $out/bin/aicentre
    runHook postInstall
  '';

  dontStrip = true;
  dontPatchELF = true;
  dontFixup = true;

  meta = {
    description = "Command line tool for P1 compute: check your setup, run sessions and jobs";
    homepage = "https://hpc.aicentre.dk/cli/";
    mainProgram = "aicentre";
    platforms = [
      "x86_64-linux"
      "aarch64-linux"
      "aarch64-darwin"
    ];
  };
}
