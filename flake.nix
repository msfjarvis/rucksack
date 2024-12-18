{
  description = "rucksack";

  inputs.nixpkgs.url = "github:msfjarvis/nixpkgs/nixpkgs-unstable";

  inputs.systems.url = "github:msfjarvis/flake-systems";

  inputs.advisory-db.url = "github:rustsec/advisory-db";
  inputs.advisory-db.flake = false;

  inputs.crane.url = "github:ipetkov/crane";

  inputs.devshell.url = "github:numtide/devshell";
  inputs.devshell.inputs.nixpkgs.follows = "nixpkgs";

  inputs.fenix.url = "github:nix-community/fenix";
  inputs.fenix.inputs.nixpkgs.follows = "nixpkgs";

  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.flake-utils.inputs.systems.follows = "systems";

  inputs.flake-compat.url = "github:nix-community/flake-compat";
  inputs.flake-compat.flake = false;

  outputs =
    {
      self,
      nixpkgs,
      advisory-db,
      crane,
      devshell,
      fenix,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ devshell.overlays.default ];
        };

        rustStable = (import fenix { inherit pkgs; }).fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-yMuSb5eQPO/bHv+Bcf/US8LVMbf/G/0MSfiPwBhiPpk=";
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustStable;
        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          buildInputs = [ ];
          nativeBuildInputs = [ ];
          cargoClippyExtraArgs = "--all-targets -- --deny warnings";
        };
        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // { doCheck = false; });

        rucksack = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
            doCheck = false;
          }
        );
        rucksack-clippy = craneLib.cargoClippy (
          commonArgs
          // {
            inherit cargoArtifacts;
          }
        );
        rucksack-fmt = craneLib.cargoFmt (commonArgs // { });
        rucksack-audit = craneLib.cargoAudit (commonArgs // { inherit advisory-db; });
        rucksack-nextest = craneLib.cargoNextest (
          commonArgs
          // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
          }
        );
      in
      {
        checks = {
          inherit
            rucksack
            rucksack-audit
            rucksack-clippy
            rucksack-fmt
            rucksack-nextest
            ;
        };

        packages.default = rucksack;

        apps.default = flake-utils.lib.mkApp { drv = rucksack; };

        devShells.default = pkgs.devshell.mkShell {
          bash = {
            interactive = "";
          };

          env = [
            {
              name = "DEVSHELL_NO_MOTD";
              value = 1;
            }
          ];

          packages = with pkgs; [
            cargo-dist
            cargo-nextest
            cargo-release
            fenix.packages.${system}.rust-analyzer
            gcc
            rustStable
            watchman
          ];
        };

        nixosModules.default =
          {
            pkgs,
            lib,
            config,
            ...
          }:
          with lib;
          let
            cfg = config.services.rucksack;
            settingsFormat = pkgs.formats.toml { };
            settingsFile = settingsFormat.generate "rucksack.toml" {
              inherit (cfg) sources target file_filter;
            };
          in
          {
            options.services.rucksack = {
              enable = mkEnableOption {
                description = mdDoc "Whether to enable the rucksack daemon.";
              };

              sources = mkOption {
                type = types.listOf types.str;
                default = [ ];
                description = mdDoc "Directories to watch and pull files from";
              };

              target = mkOption {
                type = types.str;
                default = "";
                description = mdDoc "Directory to move files from source directories";
              };

              file_filter = mkOption {
                type = types.str;
                default = "";
                description = mdDoc "Shell glob to filter files against to be eligible for moving";
              };

              package = mkPackageOptionMD pkgs "rucksack" { };

              user = mkOption {
                type = types.str;
                default = "rucksack";
                description = mdDoc "User account under which rucksack runs.";
              };

              group = mkOption {
                type = types.str;
                default = "rucksack";
                description = mdDoc "Group account under which rucksack runs.";
              };
            };

            config = mkIf cfg.enable {
              systemd.services.rucksack = {
                wantedBy = [ "default.target" ];
                after = [ "fs.service" ];
                wants = [ "fs.service" ];

                serviceConfig = {
                  User = cfg.user;
                  Group = cfg.group;
                  Restart = "on-failure";
                  RestartSec = "30s";
                  Type = "simple";
                  Environment = "PATH=${pkgs.coreutils}/bin:${pkgs.watchman}/bin";
                };
                script = ''
                  exec env RUCKSACK_CONFIG=${settingsFile} ${lib.getExe cfg.package}
                '';
              };

              users.users = mkIf (cfg.user == "rucksack") {
                rucksack = {
                  inherit (cfg) group;
                  home = cfg.dataDir;
                  createHome = false;
                  description = "rucksack daemon user";
                  isNormalUser = true;
                };
              };

              users.groups = mkIf (cfg.group == "rucksack") {
                rucksack = {
                  gid = null;
                };
              };
            };
          };
      }
    );
}
