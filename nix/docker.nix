{ inputs, self, ... }:
{
  perSystem =
    { pkgs, lib, ... }:
    let
      crossPkgs = {
        x86_64 = pkgs.pkgsCross.musl64.pkgsStatic;
        aarch64 = pkgs.pkgsCross.aarch64-multiplatform.pkgsStatic;
      };

      pkgFor =
        arch:
        crossPkgs.${arch}.callPackage ./package.nix {
          inherit self;
          inherit (inputs) nix-filter;
        };

      containerFor =
        arch:
        pkgs.dockerTools.buildImage {
          name = "valfisk";
          tag = "latest-${arch}";
          # copyToRoot = [ pkgs.dockerTools.caCertificates ];

          config.Cmd = [ (lib.getExe (pkgFor arch)) ];

          architecture = crossPkgs.${arch}.go.GOARCH;
        };
    in
    {
      legacyPackages = {
        container-x86_64 = containerFor "x86_64";
        container-aarch64 = containerFor "aarch64";
      };
    };
}
