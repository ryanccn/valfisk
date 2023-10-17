{
  naersk,
  stdenv,
  lib,
  CoreFoundation,
  Security,
  SystemConfiguration,
  IOKit,
}:
naersk.buildPackage {
  src = lib.cleanSource ./..;

  nativeBuildInputs = lib.optionals stdenv.hostPlatform.isDarwin [
    CoreFoundation
    Security
    SystemConfiguration
    IOKit
  ];

  meta = with lib; {
    mainProgram = "valfisk";
    description = "Next generation Ryanland Discord bot, written in Rust";
    homepage = "https://github.com/ryanccn/valfisk";
    maintainers = with maintainers; [getchoo ryanccn];
    licenses = licenses.agpl3Only;
  };
}
