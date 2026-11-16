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

        dontCargoUseSystemLibs = "1";
      };

      cargoLock.outputHashes = {
        "poise-0.7.0" = "sha256-GXz9Um6roX74AbzgGKEN786oCPe+DPwfxFC1lnx8hl0=";
        "serenity-0.12.5" = "sha256-1L6tlTnC2llcKYn9zpmO0FFlXnqmaa9CD5PD6YW0MXU=";
      };

      flake.legacyPackages = lib.genAttrs lib.systems.flakeExposed (
        system:
        let
          pkgsBuild = nixpkgs.legacyPackages.${system};

          dockerArchFor = {
            "x86_64" = "amd64";
            "aarch64" = "arm64";
          };

          dockerImageFor =
            arch:
            let
              systemHost = "${arch}-linux";
              pkgsHost = nixpkgs.legacyPackages.${systemHost};
            in
            pkgsBuild.dockerTools.buildLayeredImage {
              name = "valfisk";
              tag = "latest-${arch}";
              architecture = dockerArchFor.${arch};

              contents = [
                pkgsBuild.dockerTools.caCertificates
                pkgsHost.curl-impersonate
              ];

              config.Cmd = [
                (lib.getExe self.packages.${systemHost}.valfisk)
              ];
            };
        in
        lib.genAttrs' (builtins.attrNames dockerArchFor) (
          arch: lib.nameValuePair "docker-image-${arch}" (dockerImageFor arch)
        )
      );
    };
}
