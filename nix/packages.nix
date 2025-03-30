# SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
#
# SPDX-License-Identifier: AGPL-3.0-only

{ self, ... }:
{
  perSystem =
    { pkgs, config, ... }:
    {
      packages = {
        valfisk = pkgs.callPackage ./package.nix { inherit self; };
        default = config.packages.valfisk;
      };
    };
}
