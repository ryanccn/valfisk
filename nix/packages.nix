{ self, inputs, ... }:
{
  perSystem =
    { pkgs, config, ... }:
    {
      packages = {
        valfisk = pkgs.callPackage ./package.nix {
          inherit self;
          inherit (inputs) nix-filter;
        };

        default = config.packages.valfisk;
      };
    };
}
