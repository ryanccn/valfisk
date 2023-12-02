{
  self,
  inputs,
  ...
}: {
  perSystem = {
    lib,
    pkgs,
    system,
    config,
    ...
  }: {
    packages = {
      valfisk = pkgs.callPackage ./derivation.nix {
        inherit self;
        naersk = inputs.naersk.lib.${system};

        inherit (pkgs) libiconv;

        inherit
          (pkgs.darwin.apple_sdk.frameworks)
          CoreFoundation
          Security
          SystemConfiguration
          ;
      };

      default = config.packages.valfisk;
    };
  };
}
