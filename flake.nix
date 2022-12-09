{
  description = "walls-bot-rs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };

    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs =
    { self, nixpkgs, crane, flake-utils, advisory-db, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustStable = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" ];
          targets =
            pkgs.lib.optionals pkgs.stdenv.isDarwin [ "aarch64-apple-darwin" ]
            ++ pkgs.lib.optionals pkgs.stdenv.isLinux
            [ "x86_64-unknown-linux-gnu" ];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain rustStable;
        src = craneLib.cleanCargoSource ./.;
        cargoArtifacts = craneLib.buildDepsOnly { inherit src buildInputs; };
        buildInputs = [ ];

        file-collector = craneLib.buildPackage {
          inherit src;
          doCheck = false;
        };
      in {
        checks = { inherit file-collector; };

        # Run clippy (and deny all warnings) on the crate source,
        # again, resuing the dependency artifacts from above.
        #
        # Note that this is done as a separate derivation so that
        # we can block the CI if there are issues here, but not
        # prevent downstream consumers from building our crate by itself.
        file-collector-clippy = craneLib.cargoClippy {
          inherit cargoArtifacts src buildInputs;
          cargoClippyExtraArgs = "--all-targets -- --deny warnings";
        };

        file-collector-doc = craneLib.cargoDoc { inherit cargoArtifacts src; };

        # Check formatting
        file-collector-fmt = craneLib.cargoFmt { inherit src; };

        # Audit dependencies
        file-collector-audit = craneLib.cargoAudit { inherit src advisory-db; };

        # Run tests with cargo-nextest
        # Consider setting `doCheck = false` on `file-collector` if you do not want
        # the tests to run twice
        file-collector-nextest = craneLib.cargoNextest {
          inherit cargoArtifacts src buildInputs;
          partitions = 1;
          partitionType = "count";
        };

        packages.default = file-collector;

        apps.default = flake-utils.lib.mkApp { drv = file-collector; };

        devShells.default = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.checks;

          nativeBuildInputs = with pkgs; [ cargo-deny cargo-release rustStable watchman ];
        };
      });
}
