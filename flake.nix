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
        METADATA_LAST_MODIFIED = self.lastModified or 0;
      };

      doCheck = true;

      cargoLock.outputHashes = {
        "poise-0.6.1" = "sha256-qCTEkOWCpKgEXCt7apg+tiScE+X0Br0giTNNBxqNCs0=";
        "serenity-0.12.5" = "sha256-vwlSxavZ4DNGtZUvg/GuIQY8bm/OZsks0/s0lK3ZV2c=";
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

          dockerImageFor =
            arch:
            pkgs.dockerTools.buildImage {
              name = "valfisk";
              tag = "latest-${arch}";
              architecture = crossPkgs.${arch}.go.GOARCH;

              config.Cmd = [ (lib.getExe (pkgFor arch)) ];
              copyToRoot = [ pkgs.dockerTools.caCertificates ];
            };
        in
        {
          docker-image-x86_64 = dockerImageFor "x86_64";
          docker-image-aarch64 = dockerImageFor "aarch64";
        }
      );
    };
}
