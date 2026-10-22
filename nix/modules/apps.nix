{
  perSystem =
    {
      config,
      lib,
      pkgs,
      ...
    }:
    let
      inherit (config.packages.release.passthru) craneLib;
      inherit (config.packages.release.passthru.commonArgs) src;

      # Bench targets in `messagepack-bench/benches`.
      benchNames = [
        "serialization"
        "deserialization"
      ];

      # Keep these in sync with the workspace args in `nix/build.nix`.
      workspaceArgs = "--locked --workspace --exclude fuzz";

      # Derive the version from `Cargo.toml` (`workspace.package.version`) so it
      # cannot drift from the workspace.
      inherit (craneLib.crateNameFromCargoToml { cargoToml = ../../Cargo.toml; }) version;

      baseArgs = {
        inherit src;
        cargoLock = ../../Cargo.lock;
        strictDeps = true;
      };

      # The workspace `bench` profile as configured in `.cargo/config.toml`.
      benchEnv = {
        CARGO_PROFILE = "bench";
        CARGO_PROFILE_BENCH_LTO = "fat";
        CARGO_PROFILE_BENCH_CODEGEN_UNITS = "1";
      };

      # The flamegraph build additionally needs DWARF debug info and frame
      # pointers. Because Rust 1.90 links with lld by default, `--no-rosegment`
      # is required for accurate `perf` stack traces. These are scoped to this
      # build, so the benchmark and every other build stay unaffected.
      # See https://github.com/flamegraph-rs/flamegraph#linux
      flamegraphEnv = benchEnv // {
        CARGO_PROFILE_BENCH_DEBUG = "true";
        RUSTFLAGS =
          "-C force-frame-pointers=yes"
          + lib.optionalString pkgs.stdenv.hostPlatform.isLinux " -C link-arg=-Wl,--no-rosegment";
      };

      mkDeps =
        env:
        craneLib.buildDepsOnly (
          baseArgs
          // {
            inherit env;
            cargoExtraArgs = workspaceArgs;
          }
        );

      # Dependency (including dev-dependency) caches.
      benchDeps = mkDeps benchEnv;
      flamegraphDeps = mkDeps flamegraphEnv;

      # Build the benchmark executables at Nix build time (so the apps need no
      # writable state at runtime). The flamegraph variant carries debug info and
      # frame pointers for symbol resolution.
      mkBenchBundle =
        { deps, env }:
        craneLib.mkCargoDerivation (
          baseArgs
          // {
            pname = "messagepack-rs-benches";
            inherit version env;
            cargoArtifacts = deps;
            doInstallCargoArtifacts = false;
            dontStrip = true;

            buildPhaseCargoCommand = "cargo bench --no-run --locked -p messagepack-bench --benches";

            installPhaseCommand = ''
              mkdir -p $out/bin
              ${lib.concatMapStrings (name: ''
                bin="$(find "''${CARGO_TARGET_DIR:-target}/release/deps" -maxdepth 1 -type f -executable -name ${lib.escapeShellArg "${name}-*"} -print -quit)"
                if [ -z "$bin" ]; then
                  echo "could not find the '${name}' benchmark executable" >&2
                  exit 1
                fi
                cp "$bin" "$out/bin/${name}"
              '') benchNames}
            '';
          }
        );

      benchBundle = mkBenchBundle {
        deps = benchDeps;
        env = benchEnv;
      };
      flamegraphBundle = mkBenchBundle {
        deps = flamegraphDeps;
        env = flamegraphEnv;
      };

      benchWrapper = pkgs.writeShellScriptBin "messagepack-bench" ''
        set -uo pipefail
        status=0
        for name in ${lib.concatStringsSep " " benchNames}; do
          "${benchBundle}/bin/$name" --bench "$@" || status=$?
        done
        exit "$status"
      '';

      # Thin wrapper around flamegraph's `flamegraph` binary; all arguments are
      # forwarded verbatim. Point it at the prebuilt benchmark executable from
      # `packages.flamegraph`, e.g.
      #   nix run .#flamegraph -- -o out.svg -- \
      #     "$(nix build .#flamegraph --no-link --print-out-paths)/bin/deserialization" --bench
      flamegraphWrapper = pkgs.writeShellApplication {
        name = "messagepack-flamegraph";
        runtimeInputs = [
          pkgs.cargo-flamegraph
          pkgs.perf
          pkgs.coreutils
        ];
        text = ''
          exec flamegraph "$@"
        '';
      };
    in
    {
      # Benchmark executables with debug info and frame pointers, meant to be
      # profiled with `nix run .#flamegraph`.
      packages.flamegraph = flamegraphBundle;

      apps.bench = {
        type = "app";
        program = "${benchWrapper}/bin/messagepack-bench";
        meta.description = "Run the messagepack-rs benchmark suite";
      };

      apps.flamegraph = lib.mkIf pkgs.stdenv.hostPlatform.isLinux {
        type = "app";
        program = lib.getExe flamegraphWrapper;
        meta.description = "Generate a flamegraph for a prebuilt messagepack-rs benchmark";
      };
    };
}
