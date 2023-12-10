{
  self,
  inputs,
  ...
}: {
  perSystem = {
    inputs',
    lib,
    pkgs,
    system,
    config,
    ...
  }: let
    toolchain = with inputs'.fenix.packages;
      combine [
        minimal.cargo
        minimal.rustc
        minimal.rust-std
      ];

    naersk = inputs.naersk.lib.${system}.override {
      cargo = toolchain;
      rustc = toolchain;
    };
  in {
    packages = {
      valfisk = pkgs.callPackage ./derivation.nix {
        inherit self naersk;

        inherit (pkgs) libiconv;

        inherit
          (pkgs.darwin.apple_sdk.frameworks)
          CoreFoundation
          Security
          SystemConfiguration
          ;
      };

      default = config.packages.valfisk;
    };
  };
}
