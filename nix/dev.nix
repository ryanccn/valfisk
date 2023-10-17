{
  perSystem = {
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
  };
}
