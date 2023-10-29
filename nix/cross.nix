/*
mainly for internal use. builds static linux binaries
for a minimal docker image
*/
{inputs, ...}: {
  perSystem = {
    lib,
    pkgs,
    system,
    inputs',
    self',
    ...
  }: {
    legacyPackages = let
      crossPkgsFor = {
        x86_64-linux = pkgs.pkgsStatic;
        aarch64-linux = pkgs.pkgsStatic;
        x86_64-darwin = pkgs.pkgsCross.gnu64.pkgsStatic;
        aarch64-darwin = pkgs.pkgsCross.aarch64-multiplatform.pkgsStatic;
      };

      crossPkgs = crossPkgsFor.${system};
      inherit (crossPkgs.stdenv.hostPlatform) config;

      toolchain = with inputs'.fenix.packages;
        combine [
          minimal.rustc
          minimal.cargo
          # aarch64/x86_64-unknown-linux-musl don't have minimal targets :(
          (targets.${config}.minimal or targets.${config}.latest).rust-std
        ];

      naersk' = inputs.naersk.lib.${system}.override {
        cargo = toolchain;
        rustc = toolchain;
      };

      formattedConfig = lib.toUpper (builtins.replaceStrings ["-"] ["_"] config);
      linker = "${crossPkgs.stdenv.cc}/bin/${crossPkgs.stdenv.cc.targetPrefix}cc";
      valfisk = self'.packages.valfisk.override {naersk = naersk';};
    in {
      valfisk-static = valfisk.overrideAttrs (_: {
        CARGO_BUILD_TARGET = config;
        "CARGO_TARGET_${formattedConfig}_LINKER" = linker;
      });
    };
  };
}
