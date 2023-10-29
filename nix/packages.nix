{self, ...}: {
  perSystem = {
    lib,
    pkgs,
    ...
  }: {
    packages = let
      pkgs' = lib.fix (final: self.overlays.default final pkgs);
    in {
      inherit (pkgs') valfisk;
      default = pkgs'.valfisk;
    };
  };
}
