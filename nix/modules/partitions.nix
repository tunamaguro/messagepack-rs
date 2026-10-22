{ inputs, ... }:
{
  imports = [
    inputs.flake-parts.flakeModules.partitions
  ];

  partitionedAttrs = {
    checks = "dev";
    devShells = "dev";
    formatter = "dev";
  };

  partitions.dev = {
    extraInputsFlake = ../dev;

    module =
      { inputs, ... }:
      {
        imports = [
          inputs.treefmt-nix.flakeModule
          ./formatter.nix
          ./checks.nix
          ./devshells.nix
        ];
      };
  };
}
