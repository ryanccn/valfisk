{
  self,
  stdenv,
  lib,
  rustPlatform,
  pkg-config,
  libiconv,
  darwin,
  lto ? false,
  optimizeSize ? false,
}:
rustPlatform.buildRustPackage {
  pname = "valfisk";
  version = builtins.substring 0 8 self.lastModifiedDate;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../src
      ../build.rs
      ../Cargo.lock
      ../Cargo.toml
    ];
  };
  cargoLock.lockFile = ../Cargo.lock;

  __structuredAttrs = true;

  buildInputs = lib.optionals stdenv.isDarwin [
    libiconv
    darwin.apple_sdk.frameworks.CoreFoundation
    darwin.apple_sdk.frameworks.Security
    darwin.apple_sdk.frameworks.SystemConfiguration
  ];

  nativeBuildInputs = [
    pkg-config
  ];

  env =
    {
      METADATA_LAST_MODIFIED = self.lastModified;
      METADATA_GIT_REV = self.dirtyRev or self.rev;
    }
    // lib.optionalAttrs lto {
      CARGO_PROFILE_RELEASE_LTO = "fat";
    }
    // lib.optionalAttrs optimizeSize {
      CARGO_PROFILE_RELEASE_OPT_LEVEL = "z";
      CARGO_PROFILE_RELEASE_PANIC = "abort";
      CARGO_PROFILE_RELEASE_CODEGEN_UNITS = "1";
      CARGO_PROFILE_RELEASE_STRIP = "symbols";
    };

  meta = with lib; {
    mainProgram = "valfisk";
    description = "Next generation Ryanland Discord bot, written in Rust";
    homepage = "https://github.com/ryanccn/valfisk";
    maintainers = with maintainers; [getchoo ryanccn];
    licenses = licenses.agpl3Only;
  };
}
