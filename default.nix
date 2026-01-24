{
  lib,
  config,
  dream2nix,
  ...
}: {
  imports = [
    dream2nix.modules.dream2nix.rust-cargo-lock
    dream2nix.modules.dream2nix.rust-cargo-vendor
    dream2nix.modules.dream2nix.buildRustPackage
  ];

  mkDerivation = {
    src = ./.;
  };

  deps = {nixpkgs, ...}: {
    inherit
      (nixpkgs)
      stdenv
      ;
  };

  name = "declaro";
  version = "0.1.0";
}
