{ lib
, rustPlatform
, python3
, procps
, openssh
, iproute2
, stdenv
,
}:

let
  manifest = (lib.importTOML ../Cargo.toml).package;
in
rustPlatform.buildRustPackage {
  pname = "aicentre";
  inherit (manifest) version;

  src = lib.fileset.toSource {
    root = ./..;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      (lib.fileset.fileFilter (file: file.hasExt "rs" || file.name == "launcher.sh") ../src)
      ../tests
      ../install.sh
      ../release.pub
    ];
  };

  cargoLock.lockFile = ../Cargo.lock;

  nativeCheckInputs = [
    python3
    procps
    openssh
  ] ++ lib.optionals stdenv.hostPlatform.isLinux [ iproute2 ];

  checkFlags = [
    "--skip=in_a_job_a_browser_server_ends_once_no_browser_has_been_connected_for_the_idle_limit"
    "--skip=sshd_ends_the_job_once_nobody_has_been_connected_for_the_idle_limit"
  ];

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
