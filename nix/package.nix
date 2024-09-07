{
  lib,
  stdenv,
  rustPlatform,
  darwin,
  nix-filter,
  pkg-config,
  self,
  enableLTO ? true,
  enableOptimizeSize ? false,
}:
rustPlatform.buildRustPackage rec {
  pname = passthru.cargoToml.package.name;
  inherit (passthru.cargoToml.package) version;

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

  cargoLock.lockFile = ../Cargo.lock;

  doCheck = false;

  buildInputs = lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.CoreFoundation
    darwin.apple_sdk.frameworks.Security
    darwin.apple_sdk.frameworks.SystemConfiguration
    darwin.apple_sdk.frameworks.IOKit
    darwin.libiconv
  ];

  nativeBuildInputs = lib.optionals stdenv.isDarwin [ pkg-config ];

  env =
    lib.optionalAttrs enableLTO {
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
