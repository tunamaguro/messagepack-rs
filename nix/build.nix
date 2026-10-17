{
  lib,
  pkgs,
  crane,
  rustToolchainFor,
  cargoPackage ? null,
  profile ? "release",
  pname ? "messagepack-rs",
  cargoExtraArgs ? [ ],
  cargoBuildExtraArgs ? "",
  rustFlags ? null,
  doCheck ? true,
  dontStrip ? false,
  doInstallCargoArtifacts ? false,
  installCargoArtifactsMode ? "use-zstd",
}:
let
  craneLib = crane.overrideToolchain rustToolchainFor;
  rustToolchain = rustToolchainFor pkgs;

  # The `fuzz` workspace member is driven by `cargo-fuzz` and requires a
  # nightly toolchain, so it is left out of the regular Nix builds/checks.
  workspaceArgs = [
    "--locked"
    "--workspace"
    "--exclude"
    "fuzz"
  ];

  # `cleanCargoSource` drops the non-Rust files that some crates need at build
  # or test time (benchmark fixtures, trybuild snapshots), so keep those too.
  src = lib.cleanSourceWith {
    src = ../.;
    filter =
      path: type:
      let
        baseName = baseNameOf (toString path);
      in
      craneLib.filterCargoSources path type
      || lib.hasSuffix ".stderr" baseName
      || lib.hasSuffix ".txt" baseName
      || lib.hasSuffix ".md" baseName
      || baseName == "deny.toml";
  };

  commonArgs = {
    inherit src;
    cargoLock = ../Cargo.lock;
    strictDeps = true;
    inherit doCheck dontStrip;

    env = {
      CARGO_PROFILE = profile;
    }
    // lib.optionalAttrs (rustFlags != null) {
      RUSTFLAGS = rustFlags;
    };
  };

  cargoArtifacts = craneLib.buildDepsOnly (
    commonArgs
    // {
      cargoExtraArgs = lib.escapeShellArgs workspaceArgs;
      inherit cargoBuildExtraArgs;
      # Keep dev/test dependencies in the shared workspace artifact set even
      # when a final package explicitly disables its own checks.
      doCheck = true;
    }
  );

  packageCargoExtraArgs = lib.escapeShellArgs (
    workspaceArgs
    ++ lib.optionals (cargoPackage != null) [
      "-p"
      cargoPackage
    ]
    ++ cargoExtraArgs
  );

  packageArgs = commonArgs // {
    inherit
      cargoArtifacts
      cargoBuildExtraArgs
      doInstallCargoArtifacts
      installCargoArtifactsMode
      ;
    inherit pname;
    cargoExtraArgs = packageCargoExtraArgs;
  };
in
craneLib.buildPackage (
  packageArgs
  // {
    passthru = {
      inherit
        cargoArtifacts
        commonArgs
        craneLib
        rustToolchain
        ;
    };
  }
)
