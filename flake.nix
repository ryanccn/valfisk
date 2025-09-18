# SPDX-FileCopyrightText: 2025 Ryan Cao <hello@ryanccn.dev>
#
# SPDX-License-Identifier: AGPL-3.0-only

{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    ferrix.url = "github:ryanccn/ferrix";
  };

  outputs =
    {
      nixpkgs,
      ferrix,
      self,
      ...
    }@inputs:
    let
      inherit (nixpkgs) lib;
    in
    ferrix.lib.mkFlake inputs {
      root = ./.;

      env = {
        METADATA_REVISION = self.rev or self.dirtyRev or null;
      };
      doCheck = true;

      cargoLock.outputHashes = {
        "poise-0.6.1" = "sha256-4iPnHD+m+MSX4x3TyTXfoAzTCcLhvo4qDPXafvU7mAc=";
        "serenity-0.12.4" = "sha256-3inkGsZ389jbSR5DhTkkvgSbbLdYI7TesVWY8iI/fOQ=";
      };

      flake.legacyPackages = lib.genAttrs lib.systems.flakeExposed (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          crossPkgs = {
            x86_64 = pkgs.pkgsCross.musl64.pkgsStatic;
            aarch64 = pkgs.pkgsCross.aarch64-multiplatform.pkgsStatic;
          };

          pkgFor = arch: self.legacyPackages.${system}."valfisk-static-${arch}-unknown-linux-musl";

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
                '';
              destination = "/etc/os-release";
            };

          dockerImageFor =
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
          docker-image-x86_64 = dockerImageFor "x86_64";
          docker-image-aarch64 = dockerImageFor "aarch64";
        }
      );
    };
}
