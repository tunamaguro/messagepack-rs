{
  perSystem =
    { config, ... }:
    let
      rustToolchain = config.packages.release.passthru.rustToolchain;
    in
    {
      treefmt.programs = {
        rustfmt = {
          enable = true;
          package = rustToolchain;
          includes = [ "*.rs" ];
          # `trybuild` snapshots reference source lines, so leave the
          # intentional compile-failure fixtures untouched.
          excludes = [ "messagepack-derive-test/tests/fail/*.rs" ];
        };

        mdformat = {
          enable = true;
          includes = [ "*.md" ];
        };

        taplo = {
          enable = true;
          includes = [ "*.toml" ];
        };
      };
    };
}
