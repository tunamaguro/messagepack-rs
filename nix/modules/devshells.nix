{
  perSystem =
    { config, ... }:
    let
      app = config.packages.release;
    in
    {
      devShells.default = app.passthru.craneLib.devShell {
        checks = config.checks;
        inputsFrom = [ app ];
        packages = [
          config.treefmt.build.wrapper
        ];
      };
    };
}
