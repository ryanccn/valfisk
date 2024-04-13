{...}: {
  perSystem = {
    self',
    lib,
    pkgs,
    system,
    config,
    inputs',
    ...
  }: let
    crossPkgsFor = {
      x86_64 = pkgs.pkgsCross.musl64.pkgsStatic;
      aarch64 = pkgs.pkgsCross.aarch64-multiplatform.pkgsStatic;
    };

    valfiskFor = let
      toolchain = pkgs.rust-bin.stable.latest.minimal.override {
        extensions = ["rust-std"];
        targets = map (pkgs: pkgs.stdenv.hostPlatform.config) (lib.attrValues crossPkgsFor);
      };

      rustPlatforms =
        lib.mapAttrs (
          lib.const (pkgs:
            pkgs.makeRustPlatform (
              lib.genAttrs ["cargo" "rustc"] (lib.const toolchain)
            ))
        )
        crossPkgsFor;

      mkPackageWith = rustPlatform:
        self'.packages.valfisk.override {
          inherit rustPlatform;
          lto = true;
          optimizeSize = true;
        };
    in
      lib.mapAttrs' (
        target: rustPlatform:
          lib.nameValuePair target (mkPackageWith rustPlatform)
      )
      rustPlatforms;

    containerFor = arch:
      pkgs.dockerTools.buildImage {
        name = "valfisk";
        tag = "latest-${arch}";
        copyToRoot = [pkgs.dockerTools.caCertificates];
        config.Cmd = [valfiskFor.${arch}];

        architecture = crossPkgsFor.${arch}.go.GOARCH;
      };
  in {
    legacyPackages = {
      container-x86_64 = containerFor "x86_64";
      container-aarch64 = containerFor "aarch64";
    };
  };
}
