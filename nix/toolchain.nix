{ inputs, ... }:
pkgs:
let
  rustToolchainFor =
    p:
    (inputs.rust-overlay.lib.mkRustBin { } p).fromRustupToolchainFile ../rust-toolchain.toml;
in
pkgs.callPackage ./build.nix {
  crane = inputs.crane.mkLib pkgs;
  inherit rustToolchainFor;
}
