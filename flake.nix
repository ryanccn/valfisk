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

      cargoLock.outputHashes = {
        "poise-0.7.0" = "sha256-hX4Vxu0EPw2UQKCPeg12UHcmYwdPNZ4Knst66y7QmYM=";
        "serenity-0.12.5" = "sha256-tp49Roiorx2W2QrishET3nCdwx+HJ6IyqQeVQFpYC2s=";
      };

      flake.legacyPackages = lib.genAttrs lib.systems.flakeExposed (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};

          dockerArchFor = {
            "x86_64" = "amd64";
            "aarch64" = "arm64";
          };

          dockerImageFor =
            arch:
            pkgs.dockerTools.buildLayeredImage {
              name = "valfisk";
              tag = "latest-${arch}";
              architecture = dockerArchFor.${arch};

              contents = [
                pkgs.dockerTools.caCertificates
                pkgs.curl-impersonate
              ];

              config.Cmd = [
                (lib.getExe self.legacyPackages.${system}."valfisk-static-${arch}-unknown-linux-musl")
              ];
            };
        in
        lib.genAttrs' (builtins.attrNames dockerArchFor) (
          arch: lib.nameValuePair "docker-image-${arch}" (dockerImageFor arch)
        )
      );
    };
}
