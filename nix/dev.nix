{self, ...}: {
  perSystem = {
    lib,
    pkgs,
    self',
    ...
  }: {
    devShells = {
      default = pkgs.mkShell {
        packages = with pkgs; [
          rustfmt
        ];

        inputsFrom = [self'.packages.default];
      };
    };

    formatter = pkgs.alejandra;

    _module.args = {
      # helper function to evaluate valfisk using different instances of nixpkgs
      fromOverlay = p: lib.fix (final: self.overlays.default final p);
    };
  };
}
