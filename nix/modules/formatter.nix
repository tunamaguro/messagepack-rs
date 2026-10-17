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
