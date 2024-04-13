{self, ...}: {
  perSystem = {
    inputs',
    lib,
    pkgs,
    system,
    config,
    ...
  }: {
    packages = {
      valfisk = pkgs.callPackage ./derivation.nix {
        inherit self;
      };

      default = config.packages.valfisk;
    };
  };
}
