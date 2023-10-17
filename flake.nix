{
  description = "Next generation Ryanland Discord bot, written in Rust";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
    parts.url = "github:hercules-ci/flake-parts";
    parts.inputs.nixpkgs-lib.follows = "nixpkgs";
  };

  outputs = {
    self,
    parts,
    ...
  } @ inputs:
    parts.lib.mkFlake {inherit inputs;} {
      imports = [
        ./nix/cross.nix
        ./nix/docker.nix
        ./nix/dev.nix
        ./nix/overlay.nix
        ./nix/packages.nix
      ];

      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      perSystem = {
        pkgs,
        lib,
        ...
      }: {
        formatter = pkgs.alejandra;

        _module.args = {
          fromOverlay = p: lib.fix (final: self.overlays.default final p);
        };
      };
    };
}
