{
  naersk,
  stdenv,
  lib,
  CoreFoundation,
  Security,
  SystemConfiguration,
  IOKit,
  optimizeSize ? false,
}:
naersk.buildPackage {
  src = lib.cleanSource ./..;

  nativeBuildInputs = lib.optionals stdenv.hostPlatform.isDarwin [
    CoreFoundation
    Security
    SystemConfiguration
    IOKit
  ];

  RUSTFLAGS = lib.optionalString optimizeSize " -C codegen-units=1 -C strip=symbols -C opt-level=z";

  meta = with lib; {
    mainProgram = "valfisk";
    description = "Next generation Ryanland Discord bot, written in Rust";
    homepage = "https://github.com/ryanccn/valfisk";
    maintainers = with maintainers; [getchoo ryanccn];
    licenses = licenses.agpl3Only;
  };
}
