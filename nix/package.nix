# SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
#
# SPDX-License-Identifier: AGPL-3.0-only

{
  lib,
  rustPlatform,
  self,
  enableLTO ? true,
  enableOptimizeSize ? false,
}:
let
  year = builtins.substring 0 4 self.lastModifiedDate;
  month = builtins.substring 4 2 self.lastModifiedDate;
  day = builtins.substring 6 2 self.lastModifiedDate;
in
rustPlatform.buildRustPackage (finalAttrs: {
  pname = finalAttrs.passthru.cargoToml.package.name;
  version = "${finalAttrs.passthru.cargoToml.package.version}-unstable-${year}-${month}-${day}";

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../src
      ../build.rs
      ../Cargo.lock
      ../Cargo.toml
    ];
  };

  cargoLock = {
    lockFile = ../Cargo.lock;
    outputHashes = {
      "poise-0.6.1" = "sha256-hp5+EDvWJxO3H/bj4eAnMQxz6AecXFadpbSw4a4Ywc4=";
      "serenity-0.12.4" = "sha256-8JvBh9GdPu4fqiWQoK9mlJxbatdU6fTuE5WSjLnyPUk=";
    };
  };

  env =
    {
      METADATA_REVISION = self.rev or self.dirtyRev or null;
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
})
