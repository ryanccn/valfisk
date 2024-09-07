{ self, inputs, ... }:
{
  perSystem =
    { pkgs, config, ... }:
    let
      inherit (pkgs) lib;
    in
    {
      packages = {
        valfisk = pkgs.callPackage ./package.nix {
          inherit self;
          inherit (inputs) nix-filter;
        };

        default = config.packages.valfisk;
      } // (lib.attrsets.mapAttrs' (name: value: lib.nameValuePair "check-${name}" value) config.checks);
    };
}
