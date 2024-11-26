# SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
#
# SPDX-License-Identifier: AGPL-3.0-only

{
  lib,
  rustPlatform,
  pkg-config,
  nix-filter,
  self,
  enableLTO ? true,
  enableOptimizeSize ? false,
}:
let
  year = builtins.substring 0 4 self.lastModifiedDate;
  month = builtins.substring 4 2 self.lastModifiedDate;
  day = builtins.substring 6 2 self.lastModifiedDate;

  formattedDate = "${year}-${month}-${day}";
in
rustPlatform.buildRustPackage rec {
  pname = passthru.cargoToml.package.name;
  version = passthru.cargoToml.package.version + "-unstable-" + formattedDate;

  strictDeps = true;

  src = nix-filter.lib.filter {
    root = self;
    include = [
      "src"
      "build.rs"
      "Cargo.lock"
      "Cargo.toml"
    ];
  };

  cargoLock = {
    lockFile = ../Cargo.lock;
    outputHashes = {
      "poise-0.6.1" = "sha256-ZQ15F6yLpYsVER+/pWxFEZkD3LILynUClvOQRC1t3fo=";
      "serenity-0.12.4" = "sha256-HfnfSocMWreulmS9q+8/6qN8Ud1wMM9l2djfTcGGvnQ=";
    };
  };

  doCheck = false;

  nativeBuildInputs = [ pkg-config ];

  env =
    {
      METADATA_LAST_MODIFIED = self.lastModified;
      METADATA_GIT_REV = self.dirtyRev or self.rev;
    }
    // lib.optionalAttrs enableLTO {
      CARGO_PROFILE_RELEASE_LTO = "fat";
      CARGO_PROFILE_RELEASE_CODEGEN_UNITS = "1";
    }
    // lib.optionalAttrs enableOptimizeSize {
      CARGO_PROFILE_RELEASE_OPT_LEVEL = "z";
      CARGO_PROFILE_RELEASE_PANIC = "abort";
      CARGO_PROFILE_RELEASE_CODEGEN_UNITS = "1";
      CARGO_PROFILE_RELEASE_STRIP = "symbols";
    };

  passthru = {
    cargoToml = lib.importTOML ../Cargo.toml;
  };

  meta = with lib; {
    homepage = "https://github.com/ryanccn/valfisk";
    maintainers = with maintainers; [ ryanccn ];
    licenses = licenses.agpl3Only;
    mainProgram = "valfisk";
  };
}
