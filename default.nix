{
  craneLib,
  pkgs,
}:
craneLib.buildPackage {
  pname = "errornowatcher";
  version = "0.2.0";

  src = craneLib.cleanCargoSource ./.;

  nativeBuildInputs = with pkgs; [
    clang
    mold
    pkg-config
  ];

  buildInputs = with pkgs; [
    luajit
  ];
}
