{
  self,
  naersk,
  stdenv,
  lib,
  CoreFoundation,
  Security,
  SystemConfiguration,
  optimizeSize ? false,
}:
naersk.buildPackage {
  src = lib.cleanSource ./..;

  nativeBuildInputs = lib.optionals stdenv.hostPlatform.isDarwin [
    CoreFoundation
    Security
    SystemConfiguration
  ];

  RUSTFLAGS = lib.optionalString optimizeSize " -C codegen-units=1 -C strip=symbols -C opt-level=z";

  METADATA_LAST_MODIFIED = self.lastModified;
  METADATA_GIT_REV = self.dirtyRev or self.rev;

  meta = with lib; {
    mainProgram = "valfisk";
    description = "Next generation Ryanland Discord bot, written in Rust";
    homepage = "https://github.com/ryanccn/valfisk";
    maintainers = with maintainers; [getchoo ryanccn];
    licenses = licenses.agpl3Only;
  };
}
