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
      cargoBuildFlags = [ "--ignore-rust-version" ];

      cargoLock.outputHashes = {
        "poise-0.6.1" = "sha256-6NU1UOQUz8WO77Luv7VLp/RL1May65Y7JmMWxaPbgvo=";
        "serenity-0.12.5" = "sha256-j3tQkPHR1+xe8hFM8ECP04AxNPrRQpbtyv+it/7XI74=";
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

              copyToRoot = pkgs.buildEnv {
                name = "image-root";
                paths = [
                  pkgs.dockerTools.caCertificates
                  pkgs.curl-impersonate
                ];
                pathsToLink = [
                  "/bin"
                  "/etc"
                ];
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
