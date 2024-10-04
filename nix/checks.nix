{
  perSystem =
    { pkgs, config, ... }:
    let
      inherit (pkgs) lib;

      mkFlakeCheck =
        {
          name,
          nativeBuildInputs ? [ ],
          command,
          includeCargoDeps ? false,
        }:
        pkgs.stdenv.mkDerivation (
          {
            name = "check-${name}";
            inherit nativeBuildInputs;
            inherit (config.packages.valfisk) src;

            buildPhase = ''
              ${command}
              touch "$out"
            '';

            doCheck = false;
            dontInstall = true;
            dontFixup = true;
          }
          // lib.optionalAttrs includeCargoDeps {
            inherit (config.packages.valfisk) cargoDeps buildInputs;
          }
        );
    in
    {
      checks = {
        nixfmt = mkFlakeCheck {
          name = "nixfmt";
          nativeBuildInputs = with pkgs; [ nixfmt-rfc-style ];
          command = "nixfmt --check .";
        };

        rustfmt = mkFlakeCheck {
          name = "rustfmt";

          nativeBuildInputs = with pkgs; [
            cargo
            rustfmt
          ];

          command = "cargo fmt --check";
        };

        clippy = mkFlakeCheck {
          name = "clippy";
          includeCargoDeps = true;

          nativeBuildInputs = with pkgs; [
            rustPlatform.cargoSetupHook
            cargo
            rustc
            clippy
            clippy-sarif
            sarif-fmt
          ];

          command = ''
            cargo clippy --all-features --all-targets --tests \
              --offline --message-format=json \
              | clippy-sarif | tee $out | sarif-fmt
          '';
        };
      };
    };
}