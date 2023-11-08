{
  perSystem = {
    pkgs,
    config,
    ...
  }: {
    devShells = {
      default = pkgs.mkShell {
        packages = with pkgs; [
          rustfmt
          clippy
        ];

        inputsFrom = [config.packages.default];
      };
    };

    formatter = pkgs.alejandra;
  };
}
