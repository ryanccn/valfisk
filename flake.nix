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
        "poise-0.6.1" = "sha256-aVf70WF1Hr5ctgw2Gy8Xq9FbYbQh+775QPq07l8B79k=";
        "serenity-0.12.5" = "sha256-YHi8i/F82kao8TsFXloIyXayg/65k/zI1C8i3LBidHA=";
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
        lib.genAttrs' (builtins.attrNames dockerArchFor) (
          arch: lib.nameValuePair "docker-image-${arch}" (dockerImageFor arch)
        )
      );
    };
}
