{
  perSystem =
    {
      pkgs,
      config,
      ...
    }:
    {
      devShells = {
        default = pkgs.mkShell {
          packages = with pkgs; [
            config.formatter

            clippy
            rustfmt
            rust-analyzer
            libiconv
          ];

          inputsFrom = [ config.packages.valfisk ];

          env = {
            RUST_BACKTRACE = 1;
            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          };
        };
      };

      formatter = pkgs.nixfmt-rfc-style;
    };
}
