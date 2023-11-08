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
        "amd64" = pkgs.pkgsStatic;
        "arm64v8" = pkgs.pkgsCross.aarch64-multiplatform.pkgsStatic;
      };

      "aarch64-linux" = {
        "amd64" = pkgs.pkgsCross.musl64;
        "arm64v8" = pkgs.pkgsStatic;
      };

      "x86_64-darwin" = {
        "amd64" = pkgs.pkgsCross.musl64;
        "arm64v8" = pkgs.pkgsCross.aarch64-multiplatform.pkgsStatic;
      };

      "aarch64-darwin" = finalAttrs."x86_64-darwin";
    });

    nativeArchFor = {
      "amd64" = "x86_64";
      "arm64v8" = "aarch64";
    };

    valfiskFor = arch: let
      target = "${nativeArchFor.${arch}}-unknown-linux-musl";
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
        name = "ryanccn/valfisk";
        tag = "latest-${arch}";
        copyToRoot = [pkgs.dockerTools.caCertificates];
        config.Cmd = [(valfiskFor arch)];

        architecture = crossPkgsFor.${system}.${arch}.go.GOARCH;
      };
  in {
    legacyPackages = {
      container-amd64 = containerFor "amd64";
      container-arm64v8 = containerFor "arm64v8";
    };
  };
}
