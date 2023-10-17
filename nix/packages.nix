{
  perSystem = {
    pkgs,
    fromOverlay,
    ...
  }: {
    packages = let
      pkgs' = fromOverlay pkgs;
    in {
      inherit (pkgs') valfisk;
      default = pkgs'.valfisk;
    };
  };
}
