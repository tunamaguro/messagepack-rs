{ inputs, ... }:
{
  perSystem =
    { config, ... }:
    let
      package = config.packages.debug;
      inherit (package.passthru) commonArgs cargoArtifacts craneLib;
      src = commonArgs.src;
      checkArgs = commonArgs // { inherit cargoArtifacts; };
    in
    {
      checks = {
        build = config.packages.release;

        fmt = craneLib.cargoFmt {
          inherit src;
        };

        clippy = craneLib.cargoClippy (
          checkArgs
          // {
            cargoClippyExtraArgs = "--workspace --exclude fuzz --all-targets -- --deny warnings";
          }
        );

        test = craneLib.cargoTest (
          checkArgs
          // {
            cargoTestExtraArgs = "--workspace --exclude fuzz";
          }
        );

        audit = craneLib.cargoAudit {
          inherit src;
          advisory-db = inputs.advisory-db;
        };

        deny = craneLib.cargoDeny {
          inherit src;
        };
      };
    };
}
