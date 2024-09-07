{
  inputs = {
    nixpkgs.url = "nixpkgs/nixpkgs-unstable";
    parts.url = "github:hercules-ci/flake-parts";
    parts.inputs.nixpkgs-lib.follows = "nixpkgs";
    nix-filter.url = "github:numtide/nix-filter";
    flake-root.url = "github:srid/flake-root";
  };

  outputs =
    { parts, ... }@inputs:
    parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.flake-root.flakeModule
        ./nix
      ];

      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
    };
}
