# SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
#
# SPDX-License-Identifier: AGPL-3.0-only

_: {
  imports = [
    ./packages.nix
    ./docker.nix
    ./dev.nix
    ./checks.nix
  ];
}
