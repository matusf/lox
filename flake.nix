{
  description = "Lox";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [rust-overlay.overlays.default];
      };
      tester = pkgs.buildGoModule {
        pname = "tester";
        version = "64-dev";

        src = pkgs.fetchFromGitHub {
          owner = "matusf";
          repo = "interpreter-tester";
          rev = "bdfcf62";
          hash = "sha256-9bJSDFBo7fFhD1vnDeB9MV7QtmTMttkhyzVuWdfbVZI=";
        };

        vendorHash = "sha256-57Hr1A2OpbWcvAN/Z15fKpRWRvDkliOljhP35yuc+NU=";

        # Running tests require complicated setup from multiple repositories
        doCheck = false;
      };

      lox = pkgs.rustPlatform.buildRustPackage {
        pname = "lox";
        version = "0.1.0";
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;
        nativeCheckInputs = [tester];
        checkFlags = ["--" "--test-threads=1"];
      };
    in {
      packages = {
        tester = tester;
      };
      devShells.default = pkgs.mkShell {
        packages = [tester lox];
      };
    });
}
