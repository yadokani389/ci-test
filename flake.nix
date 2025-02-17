{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";
    git-hooks-nix.url = "github:cachix/git-hooks.nix";
    services-flake.url = "github:juspay/services-flake";
    process-compose-flake.url = "github:Platonic-Systems/process-compose-flake";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      flake-parts,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = with inputs; [
        treefmt-nix.flakeModule
        git-hooks-nix.flakeModule
        process-compose-flake.flakeModule
      ];
      systems = import inputs.systems;

      perSystem =
        {
          config,
          pkgs,
          system,
          ...
        }:
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ inputs.rust-overlay.overlays.default ];
          };

          devShells.default = pkgs.mkShell {
            inputsFrom = [
              config.pre-commit.devShell
            ];
            packages = with pkgs; [
              rust-bin.stable.latest.default
              process-compose
              fastfetch
            ];

            DATABASE_URL = "postgresql://localhost:5432/app?user=app&password=passwd";
          };

          treefmt = {
            projectRootFile = "flake.nix";
            programs = {
              nixfmt.enable = true;
              rustfmt.enable = true;
            };

            settings.formatter = { };
          };

          pre-commit = {
            check.enable = true;
            settings = {
              hooks = {
                ripsecrets.enable = true;
                cargo-check.enable = true;
                clippy.enable = true;
                typos.enable = true;
                treefmt.enable = true;
              };
            };
          };

          process-compose."dev" =
            let
              dbName = "app";
              dbUser = "app";
              dbPassword = "passwd";
              dbPort = 5432;
            in
            {
              imports = [
                inputs.services-flake.processComposeModules.default
              ];

              services = {
                postgres."pg1" = {
                  enable = true;
                  port = dbPort;
                  initialScript.before = ''
                    CREATE USER ${dbUser} SUPERUSER PASSWORD '${dbPassword}' CREATEDB;
                  '';
                  initialDatabases = [
                    {
                      name = dbName;
                    }
                  ];
                };
                redis."r1" = {
                  enable = true;
                  port = 6379;
                };
              };
            };
        };
    };
}
