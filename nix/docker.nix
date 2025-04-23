# SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
#
# SPDX-License-Identifier: AGPL-3.0-only

{ self, ... }:
{
  perSystem =
    { pkgs, lib, ... }:
    let
      crossPkgs = {
        x86_64 = pkgs.pkgsCross.musl64.pkgsStatic;
        aarch64 = pkgs.pkgsCross.aarch64-multiplatform.pkgsStatic;
      };

      pkgFor = arch: crossPkgs.${arch}.callPackage ./package.nix { inherit self; };

      osReleaseFor =
        arch:
        pkgs.writeTextFile {
          name = "valfisk-etc-os-release";
          text =
            let
              inherit (pkgFor arch) version;
            in
            ''
              NAME="Valfisk Linux"
              ID=valfisk
              VERSION_ID=${version}
              PRETTY_NAME="Valfisk Linux v${version}"
              HOME_URL="https://github.com/ryanccn/valfisk"
              BUG_REPORT_URL="https://github.com/ryanccn/valfisk/issues"
            '';
          destination = "/etc/os-release";
        };

      containerFor =
        arch:
        pkgs.dockerTools.buildImage {
          name = "valfisk";
          tag = "latest-${arch}";
          architecture = crossPkgs.${arch}.go.GOARCH;

          copyToRoot = pkgs.buildEnv {
            name = "valfisk-image-root-${arch}";
            paths = [ (osReleaseFor arch) ];
            pathsToLink = [ "/etc" ];
          };

          config.Cmd = [ (lib.getExe (pkgFor arch)) ];
        };
    in
    {
      legacyPackages = {
        container-x86_64 = containerFor "x86_64";
        container-aarch64 = containerFor "aarch64";
      };
    };
}
