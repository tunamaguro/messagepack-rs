{
  perSystem =
    {
      config,
      lib,
      pkgs,
      ...
    }:
    let
      inherit (config.packages.release.passthru) craneLib rustToolchain;
      inherit (config.packages.release.passthru.commonArgs) src;

      # Bench target names in `messagepack-bench/benches`.
      benchNames = [
        "serialization"
        "deserialization"
      ];

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

      # The flamegraph needs DWARF debug info to resolve symbols and, because
      # Rust 1.90 links with lld by default, the `--no-rosegment` link flag for
      # accurate `perf` stack traces. Both are scoped to the flamegraph build so
      # no other build carries them.
      # See https://github.com/flamegraph-rs/flamegraph#linux
      flamegraphEnv =
        benchEnv
        // {
          CARGO_PROFILE_BENCH_DEBUG = "true";
        }
        // lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
          RUSTFLAGS = "-Clink-arg=-Wl,--no-rosegment";
        };

      flamegraphEnvExports = lib.concatStringsSep "\n" (
        lib.mapAttrsToList (name: value: "export ${name}=${lib.escapeShellArg value}") flamegraphEnv
      );

      commonArgs = baseArgs // {
        env = benchEnv;
      };

      # Vendored sources for offline cargo invocations.
      vendorDir = craneLib.vendorCargoDeps baseArgs;

      # Dependency (including dev-dependency) cache for the benchmark binaries.
      # Keep this in sync with the workspace args in `nix/build.nix`.
      benchDeps = craneLib.buildDepsOnly (
        commonArgs
        // {
          cargoExtraArgs = "--locked --workspace --exclude fuzz";
        }
      );

      # Build the benchmark executables at Nix build time. Crane's
      # `inheritCargoArtifactsHook` restores `benchDeps` into the build, so the
      # dependency graph is not rebuilt. Only the executables are installed, so
      # the app needs no writable host state.
      benchBundle = craneLib.mkCargoDerivation (
        commonArgs
        // {
          pname = "messagepack-rs-benches";
          inherit version;
          cargoArtifacts = benchDeps;
          doInstallCargoArtifacts = false;
          # Keep symbols in `$out/bin` so `perf` can unwind them.
          dontStrip = true;

          # `-p messagepack-bench` already limits the build to that package, so
          # `--workspace --exclude fuzz` is unnecessary here.
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

      # Separate, flamegraph-only dependency cache (with debug info and the
      # `--no-rosegment` flag). Restored by the flamegraph app below. It is built
      # separately from `benchDeps` on purpose: profiling is interactive, so we
      # trade Nix build time/disk for a fast runtime restore (only the workspace
      # crates are relinked, not the whole dependency graph).
      flamegraphDeps = craneLib.buildDepsOnly (
        baseArgs
        // {
          env = flamegraphEnv;
          cargoExtraArgs = "--locked --workspace --exclude fuzz";
        }
      );

      benchWrapper = pkgs.writeShellScriptBin "messagepack-bench" ''
        set -uo pipefail
        status=0
        for name in ${lib.concatStringsSep " " benchNames}; do
          "${benchBundle}/bin/$name" --bench "$@" || status=$?
        done
        exit "$status"
      '';

      # `cargo flamegraph` needs a writable target directory, so this app
      # restores the reproducible `$out` cache into an ephemeral directory and
      # (re)builds/links against the workspace source before profiling.
      flamegraphWrapper = pkgs.writeShellScriptBin "messagepack-flamegraph" ''
        export PATH="${
          lib.makeBinPath [
            rustToolchain
            pkgs.cargo-flamegraph
            pkgs.coreutils
            pkgs.findutils
            pkgs.gnutar
            pkgs.perf
            pkgs.rsync
            pkgs.zstd
          ]
        }:$PATH"
        set -euo pipefail

        # NOTE: this sources crane's setup hook. Its path and the
        # `inheritCargoArtifacts` function are part of crane's internal
        # interface (not its public API), so keep it in sync with the crane
        # revision pinned in `flake.lock`.
        source ${craneLib.inheritCargoArtifactsHook}/nix-support/setup-hook >/dev/null

        tmp="$(mktemp -d)"
        trap 'rm -rf "$tmp"' EXIT
        mkdir -p "$tmp/target" "$tmp/cargo-home"
        NIX_BUILD_CORES="$(nproc)" inheritCargoArtifacts "${flamegraphDeps}" "$tmp/target"
        cp "${vendorDir}/config.toml" "$tmp/cargo-home/config.toml"

        export CARGO_HOME="$tmp/cargo-home"
        export CARGO_TARGET_DIR="$tmp/target"
        export CARGO_NET_OFFLINE=true
        ${flamegraphEnvExports}

        # Run from the caller's directory so the default `flamegraph.svg`
        # lands next to them; build against the (read-only) Nix source.
        existing_perf_data=0
        if [ -e perf.data ]; then existing_perf_data=1; fi
        cargo flamegraph --manifest-path ${src}/Cargo.toml "$@"
        # `cargo flamegraph` leaves `perf.data` behind; only clean up our own.
        if [ "$existing_perf_data" = 0 ]; then rm -f perf.data 2>/dev/null || true; fi
      '';
    in
    {
      apps.bench = {
        type = "app";
        program = "${benchWrapper}/bin/messagepack-bench";
        meta.description = "Run the messagepack-rs benchmark suite";
      };

      apps.flamegraph = lib.mkIf pkgs.stdenv.hostPlatform.isLinux {
        type = "app";
        program = "${flamegraphWrapper}/bin/messagepack-flamegraph";
        meta.description = "Generate a flamegraph for a messagepack-rs benchmark";
      };
    };
}
