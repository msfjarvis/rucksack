{
  description = "walls-bot-rs";

  inputs = {
    nixpkgs = {url = "github:NixOS/nixpkgs/nixpkgs-unstable";};

    fenix = {
      url = "github:nix-community/fenix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };

    flake-utils = {url = "github:numtide/flake-utils";};

    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };

    crane = {
      url = "github:ipetkov/crane";
      inputs = {
        flake-compat.follows = "flake-compat";
        flake-utils.follows = "flake-utils";
        nixpkgs.follows = "nixpkgs";
      };
    };

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = {
    self,
    nixpkgs,
    fenix,
    crane,
    flake-utils,
    advisory-db,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};

      rustStable = (import fenix {inherit pkgs;}).fromToolchainFile {
        file = ./rust-toolchain.toml;
        sha256 = "sha256-eMJethw5ZLrJHmoN2/l0bIyQjoTX1NsvalWSscTixpI=";
      };

      craneLib = (crane.mkLib pkgs).overrideToolchain rustStable;

      commonArgs = {
        src = craneLib.cleanCargoSource ./.;
        buildInputs = [];
        nativeBuildInputs = [];
        cargoClippyExtraArgs = "--all-targets -- --deny warnings";
      };

      cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {doCheck = false;});

      file-collector = craneLib.buildPackage (commonArgs
        // {
          inherit cargoArtifacts;
          doCheck = false;
        });

      file-collector-clippy = craneLib.cargoClippy (commonArgs
        // {
          inherit cargoArtifacts;
        });

      file-collector-fmt = craneLib.cargoFmt (commonArgs // {});

      file-collector-audit = craneLib.cargoAudit (commonArgs // {inherit advisory-db;});

      file-collector-nextest = craneLib.cargoNextest (commonArgs
        // {
          inherit cargoArtifacts;
          partitions = 1;
          partitionType = "count";
        });
    in {
      checks = {
        inherit file-collector file-collector-audit file-collector-clippy file-collector-fmt file-collector-nextest;
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

        CARGO_REGISTRIES_CRATES_IO_PROTOCOL = "sparse";
      };
    });
}
