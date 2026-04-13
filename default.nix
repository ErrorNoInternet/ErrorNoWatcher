{
  lib,
  pkgs,
  rust,
  self,
}:
pkgs.rustPlatform.buildRustPackage {
  pname = "errornowatcher";
  version = self.shortRev or self.dirtyShortRev;

  cargoLock.lockFile = ./Cargo.lock;
  src = lib.cleanSource ./.;

  nativeBuildInputs = with pkgs; [
    rust

    mold
    pkg-config
  ];

  buildInputs = with pkgs; [
    luajit
  ];
}
