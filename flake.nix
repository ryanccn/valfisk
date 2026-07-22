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
        "poise-0.6.1" = "sha256-a29KM2rZ9EVUuqAWqJO3FYT5VqBnLVMSf6Ucc+R6dus=";
        "serenity-0.12.5" = "sha256-Vh5iXdfYRg3dy5AxCrVU9o0G/jyXh9pLAizIVe2sEig=";
      };

      flake.legacyPackages = lib.genAttrs lib.systems.flakeExposed (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};

          dockerArchFor = {
            "x86_64" = "amd64";
            "aarch64" = "arm64";
          };
          pkgFor = arch: self.legacyPackages.${system}."valfisk-static-${arch}-unknown-linux-musl";

          dockerImageFor =
            arch:
            pkgs.dockerTools.buildImage {
              name = "valfisk";
              tag = "latest-${arch}";
              architecture = dockerArchFor.${arch};

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
