{
  inputs = {
    crane.url = "github:ipetkov/crane";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-parts.url = "github:hercules-ci/flake-parts";

    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    {
      crane,
      fenix,
      flake-parts,
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
          craneLib = (crane.mkLib pkgs).overrideToolchain fenix.packages.${system}.complete.toolchain;
        in
        {
          devShells.default = pkgs.mkShell {
            name = "errornowatcher";

            inputsFrom = [ self'.packages.default ];
            nativeBuildInputs = [ pkgs.pkg-config ];
            buildInputs = [ pkgs.taplo ];

            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
            RUST_BACKTRACE = 1;
          };

          packages = rec {
            default = errornowatcher;
            errornowatcher = pkgs.callPackage ./. { inherit craneLib; };
          };
        };
    };

  description = "A Minecraft bot with Lua scripting support";
}
