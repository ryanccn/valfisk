{
  self,
  naersk,
  stdenv,
  lib,
  libiconv,
  CoreFoundation,
  Security,
  SystemConfiguration,
  optimizeSize ? false,
  callPackage,
  librusty_v8 ? callPackage ./librusty_v8.nix {},
}:
naersk.buildPackage {
  src = lib.cleanSource ./..;

  nativeBuildInputs = lib.optionals stdenv.hostPlatform.isDarwin [
    libiconv
    CoreFoundation
    Security
    SystemConfiguration
  ];

  RUSTFLAGS = lib.optionalString optimizeSize " -C codegen-units=1 -C strip=symbols -C opt-level=z";

  METADATA_LAST_MODIFIED = self.lastModified;
  METADATA_GIT_REV = self.dirtyRev or self.rev;

  RUSTY_V8_ARCHIVE = librusty_v8;

  meta = with lib; {
    mainProgram = "valfisk";
    description = "Next generation Ryanland Discord bot, written in Rust";
    homepage = "https://github.com/ryanccn/valfisk";
    maintainers = with maintainers; [getchoo ryanccn];
    licenses = licenses.agpl3Only;
  };
}
