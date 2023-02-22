{
  description = "walls-bot-rs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-compat.follows = "flake-compat";
      inputs.flake-utils.follows = "flake-utils";
      inputs.rust-overlay.follows = "rust-overlay";
    };

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    flake-utils,
    advisory-db,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };

      rustStable = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      craneLib = (crane.mkLib pkgs).overrideToolchain rustStable;
      src = craneLib.cleanCargoSource ./.;
      cargoArtifacts = craneLib.buildDepsOnly {inherit src buildInputs;};
      buildInputs = [];

      file-collector = craneLib.buildPackage {
        inherit src;
        doCheck = false;
      };

      file-collector-clippy = craneLib.cargoClippy {
        inherit cargoArtifacts src buildInputs;
        cargoClippyExtraArgs = "--all-targets -- --deny warnings";
      };

      file-collector-fmt = craneLib.cargoFmt {inherit src;};

      file-collector-audit = craneLib.cargoAudit {inherit src advisory-db;};

      file-collector-nextest = craneLib.cargoNextest {
        inherit cargoArtifacts src buildInputs;
        partitions = 1;
        partitionType = "count";
      };
    in {
      checks = {
        inherit
          file-collector
          file-collector-audit
          file-collector-clippy
          file-collector-fmt
          file-collector-nextest
          ;
      };

      packages.default = file-collector;

      apps.default = flake-utils.lib.mkApp {drv = file-collector;};

      devShells.default = pkgs.mkShell {
        inputsFrom = builtins.attrValues self.checks;

        nativeBuildInputs = with pkgs; [
          cargo-audit
          cargo-release
          rustStable
          watchman
        ];
      };
    });
}
