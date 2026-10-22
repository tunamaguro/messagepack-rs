{ inputs, ... }:
{
  perSystem =
    {
      pkgs,
      ...
    }:
    let
      mkPackage = import ../toolchain.nix { inherit inputs; };
      app = mkPackage pkgs;
      release = app.override {
        profile = "release";
        dontStrip = false;
      };

      debug = app.override {
        profile = "dev";
        dontStrip = true;
      };
    in
    {
      packages = {
        default = release;
        inherit debug release;
      };
    };
}
