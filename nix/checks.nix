# SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
#
# SPDX-License-Identifier: AGPL-3.0-only

{ self, ... }:
{
  perSystem =
    { pkgs, config, ... }:
    let
      mkFlakeCheck =
        args:
        pkgs.stdenv.mkDerivation (
          {
            name = "check-${args.name}";
            src = config.packages.valfisk.src;

            buildPhase = ''
              ${args.command}
              touch "$out"
            '';

            doCheck = false;
            dontInstall = true;
            dontFixup = true;
          }
          // (removeAttrs args [
            "name"
            "command"
          ])
        );
    in
    {
      checks = {
        nixfmt = mkFlakeCheck {
          name = "nixfmt";
          command = "nixfmt --check **/*.nix";

          src = self;
          nativeBuildInputs = with pkgs; [ nixfmt-rfc-style ];
        };

        rustfmt = mkFlakeCheck {
          name = "rustfmt";
          command = "cargo fmt --check";

          nativeBuildInputs = with pkgs; [
            cargo
            rustfmt
          ];
        };

        clippy = mkFlakeCheck {
          name = "clippy";
          command = ''
            cargo clippy --all-features --all-targets --tests \
              --offline --message-format=json \
              | clippy-sarif | tee $out | sarif-fmt
          '';

          inherit (config.packages.valfisk) cargoDeps;
          nativeBuildInputs = with pkgs; [
            rustPlatform.cargoSetupHook
            cargo
            rustc
            clippy
            clippy-sarif
            sarif-fmt
          ];
        };

        reuse = mkFlakeCheck {
          name = "reuse";
          command = "reuse lint";

          src = self;
          nativeBuildInputs = with pkgs; [
            reuse
          ];
        };
      };
    };
}
