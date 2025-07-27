{
  description = "Double-entry plain-text accounting, envelope budgeting built in";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-flake = {
      url = "github:juspay/rust-flake";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      flake-parts,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } (
      { ... }:
      {
        imports = [
          inputs.rust-flake.flakeModules.default
          inputs.rust-flake.flakeModules.nixpkgs
          inputs.treefmt-nix.flakeModule
        ];

        systems = [
          "x86_64-linux"
          "x86_64-darwin"
          "aarch64-linux"
          "aarch64-darwin"
        ];

        flake = { };

        perSystem =
          {
            self',
            system,
            ...
          }:
          {
            # packages = self'.packages.silverfox;
            devShells.default = self'.devShells.rust;

            rust-project.toolchain = inputs.fenix.packages.${system}.complete.withComponents [
              "cargo"
              "clippy"
              "rust-src"
              "rust-std"
              "rustc"
              "rustfmt"
            ];
            treefmt.programs.rustfmt.enable = true;
          };
      }
    );
}
