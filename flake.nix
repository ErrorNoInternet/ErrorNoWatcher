{
  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";

    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      flake-parts,
      nixpkgs,
      rust-overlay,
      self,
      ...
    }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "aarch64-linux"
        "x86_64-linux"
      ];

      perSystem =
        {
          pkgs,
          self',
          system,
          ...
        }:
        let
          rust = pkgs.rust-bin.nightly.latest.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer-preview"
            ];
          };
        in
        {
          _module.args.pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };

          devShells.default = pkgs.mkShell {
            name = "errornowatcher";

            inputsFrom = [ self'.packages.default ];
            nativeBuildInputs = [ pkgs.pkg-config ];
            buildInputs = [ pkgs.taplo ];

            RUST_BACKTRACE = 1;
          };

          packages = rec {
            default = errornowatcher;
            errornowatcher = pkgs.callPackage ./. { inherit rust self; };
          };
        };
    };

  description = "A Minecraft bot with Lua scripting support";
}
