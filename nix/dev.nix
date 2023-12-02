{
  perSystem = {
    lib,
    pkgs,
    config,
    ...
  }: {
    /*
    You can run `daemons` in the devShell to launch these;
    `REDIS_URL` should be set to `redis://127.0.0.1`
    */
    proc.groups.daemons.processes = {
      redis.command = lib.getExe' pkgs.redis "redis-server";
    };

    devShells = {
      default = pkgs.mkShell {
        packages = with pkgs; [
          config.formatter
          config.proc.groups.daemons.package

          cargo
          rustc
          clippy
          rustfmt
          rust-analyzer
          libiconv
        ];

        RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
      };
    };

    formatter = pkgs.alejandra;
  };
}
