{
  description = "Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-darwin"];

      imports = [
        inputs.treefmt-nix.flakeModule
      ];

      perSystem = {
        pkgs,
        system,
        ...
      }: let
        overlay = final: prev: let
          rustc = prev.rustc;
          cargo = prev.cargo;
          rust-analyzer = prev.rust-analyzer;
          clippy = prev.clippy;
          rustfmt = prev.rustfmt;
        in {
          inherit rustc cargo rust-analyzer clippy rustfmt;
        };
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [overlay];
        };
      in {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustc
            cargo
            rust-analyzer
            clippy
            rustfmt
            cargo-watch
            cargo-edit
          ];
        };

        treefmt = {
          projectRootFile = "flake.nix";
          programs = {
            rustfmt.enable = true;
            alejandra.enable = true;
          };
        };
      };
    };
}
