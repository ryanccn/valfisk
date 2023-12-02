{inputs, ...}: {
  perSystem = {
    lib,
    pkgs,
    system,
    config,
    ...
  }: {
    packages = {
      valfisk = pkgs.callPackage ./derivation.nix {
        naersk = inputs.naersk.lib.${system};

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
