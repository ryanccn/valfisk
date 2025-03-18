# SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
#
# SPDX-License-Identifier: AGPL-3.0-only

{
  lib,
  rustPlatform,
  nix-filter,
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
      "poise-0.6.1" = "sha256-2MVrgJtBS2yaWlVezwsVrvDlqpfPf7fWKPfAXrqFKcA=";
      "serenity-0.12.4" = "sha256-aZ7Wgd81LztRO0Ypte8NdOnsg2gveY9wG5v33NQco1s=";
    };
  };

  doCheck = false;

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
})
