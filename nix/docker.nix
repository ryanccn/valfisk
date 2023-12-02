{inputs, ...}: {
  perSystem = {
    lib,
    pkgs,
    system,
    config,
    inputs',
    ...
  }: let
    crossPkgsFor = lib.fix (finalAttrs: {
      "x86_64-linux" = {
        "x86_64" = pkgs.pkgsStatic;
        "aarch64" = pkgs.pkgsCross.aarch64-multiplatform.pkgsStatic;
      };

      "aarch64-linux" = {
        "x86_64" = pkgs.pkgsCross.musl64;
        "aarch64" = pkgs.pkgsStatic;
      };

      "x86_64-darwin" = {
        "x86_64" = pkgs.pkgsCross.musl64;
        "aarch64" = pkgs.pkgsCross.aarch64-multiplatform.pkgsStatic;
      };

      "aarch64-darwin" = finalAttrs."x86_64-darwin";
    });

    valfiskFor = arch: let
      target = "${arch}-unknown-linux-musl";
      target' = builtins.replaceStrings ["-"] ["_"] target;
      targetUpper = lib.toUpper target';

      toolchain = with inputs'.fenix.packages;
        combine [
          minimal.cargo
          minimal.rustc
          targets.${target}.latest.rust-std
        ];

      naersk' = inputs.naersk.lib.${system}.override {
        cargo = toolchain;
        rustc = toolchain;
      };

      valfisk = config.packages.valfisk.override {
        naersk = naersk';
        optimizeSize = true;
      };

      inherit (crossPkgsFor.${system}.${arch}.stdenv) cc;
    in
      lib.getExe (
        valfisk.overrideAttrs (_:
          lib.fix (finalAttrs: {
            CARGO_BUILD_TARGET = target;
            "CC_${target'}" = "${cc}/bin/${cc.targetPrefix}cc";
            "CARGO_TARGET_${targetUpper}_RUSTFLAGS" = "-C target-feature=+crt-static";
            "CARGO_TARGET_${targetUpper}_LINKER" = finalAttrs."CC_${target'}";
          }))
      );

    containerFor = arch:
      pkgs.dockerTools.buildImage {
        name = "valfisk";
        tag = "latest-${arch}";
        copyToRoot = [pkgs.dockerTools.caCertificates];
        config.Cmd = [(valfiskFor arch)];

        architecture = crossPkgsFor.${system}.${arch}.go.GOARCH;
      };
  in {
    legacyPackages = {
      container-x86_64 = containerFor "x86_64";
      container-aarch64 = containerFor "aarch64";
    };
  };
}
